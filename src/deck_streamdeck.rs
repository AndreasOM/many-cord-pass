use derivative::Derivative;

use crate::deck::Deck;

#[derive(Derivative, Default)]
#[derivative(Debug)]
pub struct Deck_Streamdeck {
    #[derivative(Debug = "ignore")]
    streamdeck: Option<streamdeck::StreamDeck>,
}

fn find_deck() -> anyhow::Result<(u16, u16, Option<String>)> {
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

    Ok((
        device.vendor_id(),
        device.product_id(),
        device.serial_number().map(|str| String::from(str)),
    ))
}

impl Deck_Streamdeck {
    pub fn find_and_connect(/* :TODO: optional name? */) -> anyhow::Result<(Deck_Streamdeck)> {
        let (vid, pid, serial) = find_deck()?;

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

        Ok(d)
    }
}

impl Deck for Deck_Streamdeck {
    fn set_button_file(&mut self, index: u8, filename: &str) -> anyhow::Result<()> {
        if let Some(streamdeck) = &mut self.streamdeck {
            let opts = streamdeck::images::ImageOptions::default();
            streamdeck.set_button_file(index, &filename, &opts)?;
        }
        Ok(())
    }

    fn read_buttons(&mut self, timeout: Option<std::time::Duration>) -> anyhow::Result<Vec<u8>> {
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
    }

    fn set_button_rgb(&mut self, index: u8, r: u8, g: u8, b: u8) -> anyhow::Result<()> {
        if let Some(streamdeck) = &mut self.streamdeck {
            let c = streamdeck::Colour { r, g, b };
            streamdeck.set_button_rgb(index, &c);
            Ok(())
        } else {
            Err(anyhow::anyhow!("No Streamdeck to set button colors!"))
        }
    }
}
