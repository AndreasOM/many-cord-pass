
use std::collections::HashMap;

use derivative::Derivative;

use crate::action::Action;
use crate::Button;
use crate::Config;
use crate::Page;

#[derive(Derivative,Default)]
#[derivative(Debug)]
pub struct ManyCordPass {
	config:		Option< Config >,
	buttons:	HashMap< String, Button >,
	pages:		Vec< Page >,
	active_page: usize,
	#[derivative(Debug="ignore")]	
	deck:		Option< streamdeck::StreamDeck >,
	pressed_buttons:	Vec<bool>,
	done:		bool,
}


fn find_deck() -> (u16, u16, Option<String>) {
    let hid = hidapi::HidApi::new().expect("could not connect to hidapi");
    let device = hid
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
        .expect("Could not find Streamdeck");

    println!(
        "Attempting to connect to {:?}. vid: {:?}, pid: {:?}, serial: {:?}",
        device.product_string(),
        device.vendor_id(),
        device.product_id(),
        device.serial_number(),
    );

    (
        device.vendor_id(),
        device.product_id(),
        device.serial_number().map(|str| String::from(str)),
    )
}

impl ManyCordPass {
	pub fn load_config(&mut self, filename: &str) -> anyhow::Result<()> {
		let config = Config::from_file( "config.yaml" )?;
		self.config = Some( config );

		Ok(())
	}

	pub fn find_and_connect( &mut self ) -> anyhow::Result<()> {

	    let (vid, pid, serial) = find_deck();

	    let mut deck = match streamdeck::StreamDeck::connect(vid, pid, serial) {
	        Ok(d) => d,
	        Err(e) => {
	            println!("Error connecting to streamdeck: {:?}", e);
	            return Err(anyhow::anyhow!("Error"));
	        }
	    };

	    let version = deck.version()?;
	    println!("Firmware Version: {}", &version);

	    if let Some( config ) = &self.config {
	    	for bc in config.buttons() {
	    		println!("Button: {:?}", &bc);
	    		let mut button = Button::from_config( &bc )?;

	    		self.buttons.insert( bc.name.clone(), button );
	    	}

	    	for pc in config.pages() {
	    		let mut page = Page::from_config( &pc )?;

	    		self.pages.push( page );
	    	}

	    	println!("{:#?}", &self);
	    }

	    /*
		match deck.read_buttons(Some(std::time::Duration::from_millis(60))) {
            Ok(buttons) => {
            	self.pressed_buttons.resize( buttons.len(), false );
            	println!("Deck has {} buttons", buttons.len());
            },
            Err( e ) => {
            	self.pressed_buttons.resize( 32, false );
            },
        }
        */

		self.pressed_buttons.resize( 32, false );


	    self.deck = Some( deck );

		Ok(())
	}

	pub fn done( &self ) -> bool {
		self.done
	}

	pub fn update( &mut self ) -> anyhow::Result<()> {
		if let Some( deck ) = &mut self.deck {
			for b in self.buttons.values_mut() {
				b.update();
			};
			if let Some( page ) = self.pages.get( self.active_page ) {
				let mut index = 0;
				for b in page.buttons() {
					if let Some( bn ) = b {
						if let Some( button ) = &mut self.buttons.get( bn ) {
							if let Some( image ) = button.image() {
//								println!("Button {} -> {} ( {:?} )", index, image, button );
							    let opts = streamdeck::images::ImageOptions::default();
			                    deck.set_button_file(index, &image, &opts)?;
							}
						}
					}
					index += 1;
				}
				match deck.read_buttons(Some(std::time::Duration::from_millis(60))) {
            		Ok(buttons) => {
            			println!("{:?}", buttons);
            			let mut i = 0;
            			for b in buttons {
            				let new_state = b > 0;
            				let last_state = *self.pressed_buttons.get( i ).unwrap_or( &false );
            				if let Some( button_name ) = &page.buttons().get( i ) {
	            				if let Some( button_name ) = button_name {
									if let Some( button ) = &mut self.buttons.get_mut( button_name ) {
										let empty_actions = Vec::new();
			            				let actions = if new_state && !last_state{
			            					println!("Button {} pressed", i);
			            					button.press()
			            				} else if !new_state && last_state {
			            					println!("Button {} released", i);
			            					button.release()
			            				} else {
			            					&empty_actions
			            				};

			            				for action in actions {
				            				match action {
				            					Action::None => {},
				            					Action::Clear( r, g, b ) => {
													let c = streamdeck::Colour { r: *r, g: *g, b: *b };
													for k in 0..=14 {
													    deck.set_button_rgb(k, &c);
//													    std::thread::sleep(std::time::Duration::from_millis(delay));
													}
				            					},
				            					Action::Shutdown => {
				            						self.done = true;
				            					},
				            				}
				            			}
		            				}
		            			}
            				}
            				self.pressed_buttons[ i ] = b > 0;

            				i += 1;
            			}
//            			println!("---");
            		},
            		Err( e ) => return Err( anyhow::anyhow!("Error reading buttons {:?}", e ) ),
            	}
			}
		}
		Ok(())
	}
}
