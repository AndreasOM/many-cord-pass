use std::collections::HashMap;
use std::net::{SocketAddrV4, UdpSocket};
use std::str::FromStr;

use rosc::encoder;
use rosc::{OscMessage, OscPacket};

//use derivative::Derivative;
use crate::action::Action;
use crate::deck_minifb::DeckMinifb;
use crate::deck_streamdeck::DeckStreamdeck;
use crate::Button;
use crate::Config;
use crate::Deck;
use crate::Page;
use crate::Terminal;

//#[derive(Derivative,Default)]
//#[derivative(Debug)]
#[derive(Default)]
pub struct ManyCordPass {
	config:          Option<Config>,
	buttons:         HashMap<String, Button>,
	pages:           Vec<Page>,
	active_page:     usize,
	//	#[derivative(Debug="ignore")]
	streamdeck:      Option<streamdeck::StreamDeck>,
	terminal:        Option<Terminal>,
	//	deck:		Option< Deck_Minifb >, // :TODO: impl Deck
	deck:            Vec<Box<dyn Deck>>, // :TODO: impl Deck
	pressed_buttons: Vec<bool>,
	done:            bool,
	udp_socket:      Option<UdpSocket>,
}

impl core::fmt::Debug for ManyCordPass {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> core::fmt::Result {
		f.debug_struct("ManyCordPass")
			.field("config", &format!("{:?}", &self.config))
			// :TODO: other fields
			.finish()
	}
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
		},
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

impl ManyCordPass {
	pub fn load_config(&mut self, filename: &str) -> anyhow::Result<()> {
		let config = Config::from_file(filename)?;
		self.config = Some(config);

		Ok(())
	}

	pub fn run(&mut self) -> anyhow::Result<()> {
		match DeckStreamdeck::find_and_connect() {
			Ok(d) => {
				self.deck.push(Box::new(d));
			},
			Err(e) => {
				eprintln!(
					"Error finding streamdeck: {:?}\n\tUsing fake Minifb deck",
					&e
				);
			},
		};

		let force_fake = true;
		if force_fake || self.deck.is_empty() {
			let d = DeckMinifb::new("The Deck", 5, 3);

			self.deck.push(Box::new(d));
		}

		for d in self.deck.iter_mut() {
			d.run()?;
		}

		self.apply_config()?;

		Ok(())
	}

	fn apply_config(&mut self) -> anyhow::Result<()> {
		if let Some(config) = &self.config {
			for bc in config.buttons() {
				println!("Button: {:?}", &bc);
				let button = Button::from_config(&bc)?;

				self.buttons.insert(bc.name.clone(), button);
			}

			for pc in config.pages() {
				let page = Page::from_config(&pc)?;

				self.pages.push(page);
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

		self.pressed_buttons.resize(32, false);

		Ok(())
	}
	pub fn find_and_connect(&mut self) -> anyhow::Result<()> {
		let (vid, pid, serial) = find_deck()?;

		let mut streamdeck = match streamdeck::StreamDeck::connect(vid, pid, serial) {
			Ok(d) => d,
			Err(e) => {
				println!("Error connecting to streamdeck: {:?}", e);
				return Err(anyhow::anyhow!("Error"));
			},
		};

		let version = streamdeck.version()?;
		println!("Firmware Version: {}", &version);

		self.apply_config()?;

		self.streamdeck = Some(streamdeck);

		Ok(())
	}

	pub fn enable_terminal_input(&mut self) {
		let mut terminal = Terminal::default();

		let _ = terminal.run();

		self.terminal = Some(terminal);
	}

	pub fn done(&self) -> bool {
		if self.deck.iter().any(|d| d.done()) {
			true
		} else {
			self.done
		}
	}

	pub fn update(&mut self) -> anyhow::Result<()> {
		for b in self.buttons.values_mut() {
			b.update();
		}

		let mut all_actions: Vec<Action> = Vec::new();

		for deck in self.deck.iter_mut() {
			//        if let Some(deck) = &mut self.deck {
			deck.update()?;

			if let Some(page) = self.pages.get(self.active_page) {
				let mut index = 0;
				for b in page.buttons() {
					if let Some(bn) = b {
						if let Some(button) = &mut self.buttons.get(bn) {
							if let Some(image) = button.image() {
								//									println!("Button {} -> {} ( {:?} )", index, image, button );
								//                                    let opts = streamdeck::images::ImageOptions::default();
								//                                    streamdeck.set_button_file(index, &image, &opts)?;
								deck.set_button_file(index, &image)?;
							}
						}
					}
					index += 1;
				}
				match deck.read_buttons(Some(std::time::Duration::from_millis(6))) {
					Ok(buttons) => {
						println!("{:?}", buttons);
						let mut i = 0;
						for b in buttons {
							let new_state = b > 0;
							let last_state = *self.pressed_buttons.get(i).unwrap_or(&false);
							if let Some(button_name) = &page.buttons().get(i) {
								if let Some(button_name) = button_name {
									if let Some(button) = &mut self.buttons.get_mut(button_name) {
										let empty_actions = Vec::new();
										let actions = if new_state && !last_state {
											println!("Button {} pressed", i);
											button.press()
										} else if !new_state && last_state {
											println!("Button {} released", i);
											button.release()
										} else {
											&empty_actions
										};
										for a in actions {
											//let a = a.clone();
											all_actions.push(a.clone());
										}

										//all_actions.append( actions.clone() );
									}
								}
							}
							self.pressed_buttons[i] = b > 0;

							i += 1;
						}
						//            			println!("---");
					},
					Err(_e) => {
						// return Err(anyhow::anyhow!("Error reading buttons {:?}", e)); // Not an error, just nothing happened
					},
				}
			}
			//}
		}

		for action in all_actions {
			match action {
				Action::None => {},
				Action::Clear(r, g, b) => {
					for k in 0..=14 {
						for deck in self.deck.iter_mut() {
							deck.set_button_rgb(k, r, g, b)?;
						}
					}
				},
				Action::Shutdown => {
					self.done = true;
				},
				Action::ObsScene(ip, scene) => {
					// :HACK:
					tokio::spawn(async move {
						let scene = urlencoding::decode(&scene).unwrap();
						println!("Trying to switch OBS scene on {} to >{}<", &ip, &scene);
						let client = match obws::Client::connect(
							&ip,
							4455,
							std::env::var("OBS_PASSWORD").ok(),
						)
						.await
						{
							Ok(c) => c,
							_ => panic!(),
						};

						let scene_list = client.scenes().list().await.unwrap();
						dbg!(&scene_list);
						client
							.scenes()
							.set_current_program_scene(&scene)
							.await
							.unwrap();
					});
				},
				Action::HttpGet(url) => {
					let url = url.clone();
					println!("Http Get -> {}", url);
					//                                                    let resp = reqwest::blocking::get( url )?;
					tokio::spawn(async move {
						match reqwest::get(url).await {
							Ok(resp) => {
								println!("{:#?}", resp);
								match resp.text().await {
									Ok(text) => {
										println!("{:?}", text);
									},
									Err(e) => {
										println!("Error: Http get text got: {:?}", e);
									},
								}
							},
							Err(e) => {
								println!("Error: Http get got: {:?}", e);
								//panic!("{:?}", e);
							},
						};
					});
				},
				Action::OscSend(host, msg) => {
					if self.udp_socket.is_none() {
						let addr = "0.0.0.0:3456";
						let sock_addr = SocketAddrV4::from_str(addr).unwrap();
						let sock = UdpSocket::bind(sock_addr).unwrap();
						self.udp_socket = Some(sock);
					}
					if let Some(sock) = &self.udp_socket {
						let msg_buf = encoder::encode(&OscPacket::Message(OscMessage {
							addr: msg,
							args: vec![],
						}))
						.unwrap();
						println!("Sending OSC {:?} to {}", &msg_buf, &host);
						let host_addr = SocketAddrV4::from_str(&host).unwrap();
						sock.send_to(&msg_buf, host_addr).unwrap();
					}
				},
			}
		}

		Ok(())
	}
}
