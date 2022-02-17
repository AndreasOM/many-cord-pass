use std::sync::mpsc;

use derivative::Derivative;

use crate::deck::Deck;

#[derive(Debug)]
enum MessageToDeck {
    Shutdown,
    SetButtonFile {
        index: u8,
        filename: String,
    },
    SetButtonRgb {
        index: u8,
        r: u8, g: u8, b: u8,
    },
}

#[derive(Debug)]
enum MessageFromDeck {
    Buttons( Vec< u8 > ),
}

#[derive(Debug, Default)]
struct ConnectionInfo {
    pub vid: u16,
    pub pid: u16,
    pub serial: Option< String >,
}

#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub enum ButtonContent {
    None,
    Image { name: String },
    Rgb(u8, u8, u8),
}

#[derive(Derivative, Default)]
#[derivative(Debug)]
pub struct Deck_Streamdeck {
//    #[derivative(Debug = "ignore")]
//    streamdeck: Option<streamdeck::StreamDeck>,
    connection_info: Option<ConnectionInfo>,
    button_contents: Vec<ButtonContent>,
    buttons: Vec<u8>,
    button_changed: bool,
    to_deck_tx: Option< mpsc::Sender< MessageToDeck > >,
    from_deck_rx: Option< mpsc::Receiver< MessageFromDeck > >,
}

fn find_deck() -> anyhow::Result<ConnectionInfo> {
    let hid = hidapi::HidApi::new().expect("could not connect to hidapi");
    let device = match hid
        .device_list()
        //.filter(|item| item.product_id() == 0x006d)
        .filter(|item| {
            if let Some(product) = &item.product_string() {
                dbg!(&product);
                product.contains("Stream Deck")
            } else {
                false
            }
            //dbg!(&item); item.product_id() == 0x006d
        })
        .next()
    {
        Some(d) => d,
        None => {
            return Err(anyhow::anyhow!("Could not find Stream Deck"));
        }
    };
    //        .expect("Could not find Streamdeck");

    println!(
        "Attempting to connect to {:?}. vid: {:?}, pid: {:?}, serial: {:?}",
        device.product_string(),
        device.vendor_id(),
        device.product_id(),
        device.serial_number(),
    );

    Ok( ConnectionInfo {
        vid: device.vendor_id(),
        pid: device.product_id(),
        serial: device.serial_number().map(|str| String::from(str)),
    })
}

impl Deck_Streamdeck {
    pub fn find_and_connect(/* :TODO: optional name? */) -> anyhow::Result<(Deck_Streamdeck)> {
        let connection_info = find_deck()?;
/*
        let mut streamdeck = match streamdeck::StreamDeck::connect(vid, pid, serial) {
            Ok(d) => d,
            Err(e) => {
                println!("Error connecting to streamdeck: {:?}", e);
                return Err(anyhow::anyhow!("Error"));
            }
        };

        let version = streamdeck.version()?;
        println!("Firmware Version: {}", &version);

        let mut d = Deck_Streamdeck::default();

        d.streamdeck = Some(streamdeck);
*/
        let mut d = Deck_Streamdeck::default();
        d.connection_info = Some( connection_info );

        Ok(d)
    }
}

impl Deck for Deck_Streamdeck {
    fn update(&mut self) -> anyhow::Result<()> {
        self.button_changed = false;

        if let Some( rx ) = &self.from_deck_rx {
            match rx.try_recv() {
                Ok( m ) => {
                    match m {
                        MessageFromDeck::Buttons( b ) => {
//                            println!("Buttons {:?}", &b );
                            self.buttons = b;
                            self.button_changed = true;
                        },
                        u => {
                            println!("Unhandled {:?}", u );
                        },
                    }
                }
                Err( _ ) => {},
            }
        }

        Ok(())
    }
    fn run(&mut self) -> anyhow::Result<()> {

        let button_count = 15; // :TODO: get from model
        self.button_contents
            .resize(button_count, ButtonContent::None);

        let ( tx, rx ) = mpsc::channel();
        let ( tx2, rx2 ) = mpsc::channel();

        self.to_deck_tx = Some( tx );
        self.from_deck_rx = Some( rx2 );
        if let Some( mut connection_info ) = self.connection_info.take() { // :TODO: maybe we can copy the info instead
            tokio::spawn(async move {
                // connection_info
                // rx
                // tx2
                let ci = connection_info;
                let mut streamdeck = match streamdeck::StreamDeck::connect(ci.vid, ci.pid, ci.serial) {
                    Ok(d) => d,
                    Err(e) => {
                        println!("Error connecting to streamdeck: {:?}", e);
                        return; // :TODO: Err(anyhow::anyhow!("Error"));
                    }
                };
                let version = streamdeck.version();
                match version {
                    Ok( version ) => {
                        println!("Firmware Version: {}", &version);
                    },
                    Err( _ ) => {
                        return;
                    },
                }

                loop {
                    match streamdeck.read_buttons(Some(std::time::Duration::from_millis(15))) {
                        Ok(b) => {
                            tx2.send( MessageFromDeck::Buttons( b.clone() ) );
                        },
                        Err(e) => {
//                            println!("Error reading buttons from Streamdeck: {:?}",e);
                        },
                    }

                    // :TODO: handle more than one message?!
                    let mut wait = 15;
                    let max_messages = 20;
                    for _n in 0..max_messages {
                        match rx.recv_timeout(std::time::Duration::from_millis(wait)) {
                            Ok( m ) => {
                                match m {
                                    MessageToDeck::Shutdown => {
                                        println!("Shutdown!!!!");
                                        return;
                                    },
                                    MessageToDeck::SetButtonFile{ index, ref filename } => {
                                        let opts = streamdeck::images::ImageOptions::default();
                                        streamdeck.set_button_file(index, &filename, &opts); // :TODO: ?
                                    },
                                    MessageToDeck::SetButtonRgb{ index, r, g, b } => {
                                        let c = streamdeck::Colour { r, g, b };
                                        streamdeck.set_button_rgb(index, &c);
                                    },
                                    u => {
                                        println!("Unhandled message: {:?}", u );
                                    },
                                }
                            },
                            Err( mpsc::RecvTimeoutError::Disconnected ) => {
                                println!("Disconnected!");
                                return;
                            }
                            Err( e ) => {
                                break;
                                // just timeouts
    //                            println!("Error: {:?}", &e);
                            },
                        }
                        wait = 0;
                    }
                }
            });
        };

        Ok(())
    }

    fn set_button_file(&mut self, index: u8, filename: &str) -> anyhow::Result<()> {
        let index = index as usize;
        if let Some(bc) = &self.button_contents.get(index) {
            match bc {
                ButtonContent::Image { name } => {
                    if name == filename {
                        return Ok(());
                    }
                }
                _ => {}
            }
        }

        self.button_contents[index] = ButtonContent::Image {
            name: filename.to_string(),
        };

        let index = index as u8;
        if let Some( sender ) = &mut self.to_deck_tx {
            sender.send( MessageToDeck::SetButtonFile{ index, filename: filename.to_string() } )?;
        }
        Ok(())
    }

    fn read_buttons(&mut self, timeout: Option<std::time::Duration>) -> anyhow::Result<Vec<u8>> {
        /*
        if let Some(streamdeck) = &mut self.streamdeck {
            match streamdeck.read_buttons(timeout) {
                Ok(b) => Ok(b.clone()),
                Err(e) => Err(anyhow::anyhow!(
                    "Error reading buttons from Streamdeck: {:?}",
                    e
                )),
            }
        } else {
            Err(anyhow::anyhow!("No Streamdeck to read buttons from!"))
        }
        */
        if self.button_changed {
            Ok(self.buttons.clone())
        } else {
            Err(anyhow::anyhow!("No buttons changed."))
        }
    }

    fn set_button_rgb(&mut self, index: u8, r: u8, g: u8, b: u8) -> anyhow::Result<()> {
        let index = index as usize;

        self.button_contents[index] = ButtonContent::Rgb( r, g, b );

        let index = index as u8;
        if let Some( sender ) = &mut self.to_deck_tx {
            sender.send( MessageToDeck::SetButtonRgb{ index, r, g, b } )?;
        }
        Ok(())
    }
}

impl Drop for Deck_Streamdeck {
    fn drop(&mut self) {
        if let Some( sender ) = &mut self.to_deck_tx {
//            panic!("Drop!");
            sender.send( MessageToDeck::Shutdown );
        }        
    }
}
