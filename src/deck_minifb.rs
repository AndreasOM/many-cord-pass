use std::collections::HashMap;

use derivative::Derivative;
use image::DynamicImage;
use image::GenericImageView;
use minifb::{Key, MouseButton, MouseMode, Window, WindowOptions};

use crate::deck::Deck;

#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub enum ButtonContent {
	None,
	Image { name: String },
	Rgb(u8, u8, u8),
}

#[derive(Derivative, Default)]
#[derivative(Debug)]
pub struct Deck_Minifb {
	title:           String,
	width:           usize, // in buttons
	height:          usize, // in buttons
	buffer:          Vec<u32>,
	window:          Option<Window>,
	done:            bool,
	images:          HashMap<String, DynamicImage>,
	button_contents: Vec<ButtonContent>,
	buttons:         Vec<u8>,
	button_changed:  bool,
}

const BUTTON_SIZE: usize = 72;

impl Deck_Minifb {
	pub fn new(title: &str, w: usize, h: usize) -> Self {
		Self {
			title:           title.to_string(),
			width:           w,
			height:          h,
			buffer:          Vec::new(),
			window:          None,
			done:            false,
			images:          HashMap::new(),
			button_contents: Vec::new(),
			buttons:         Vec::new(),
			button_changed:  false,
		}
	}
}

impl Deck for Deck_Minifb {
	fn run(&mut self) -> anyhow::Result<()> {
		let pw = BUTTON_SIZE * self.width;
		let ph = BUTTON_SIZE * self.height;

		self.buffer = vec![0; pw * ph];

		let mut window = Window::new(&self.title, pw, ph, WindowOptions::default())?;
		window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

		self.window = Some(window);

		self.button_contents
			.resize((self.width * self.height).into(), ButtonContent::None);
		self.buttons.resize((self.width * self.height).into(), 0);

		Ok(())
	}

	fn update(&mut self) -> anyhow::Result<()> {
		let pw = BUTTON_SIZE * self.width;
		let ph = BUTTON_SIZE * self.height;

		// render

		for (i, bc) in self.button_contents.iter().enumerate() {
			match bc {
				ButtonContent::Image { name } => {
					let x = i.wrapping_rem(self.width as usize);
					let y = (i / self.width as usize); //.floor();
								   //        			println!("Rendering {} ({}, {}) -> {}", i, x, y, &name);

					let px = BUTTON_SIZE * x;
					let py = BUTTON_SIZE * y;
					let w = BUTTON_SIZE * self.width;
					//        			let h = BUTTON_SIZE * self.height;

					if let Some(img) = &mut self.images.get(name) {
						// :TODO: this is very inefficient -> fix
						for y in 0..BUTTON_SIZE {
							for x in 0..BUTTON_SIZE {
								let pixel = img.get_pixel(x as u32, y as u32);

								let pixel: u32 =
									(((pixel[3] & 0xff) as u32) << 24)
										| (((pixel[0] & 0xff) as u32) << 16) | (((pixel[1] & 0xff)
										as u32) << 8) | (((pixel[2] & 0xff) as u32) << 0);

								let o = ((py + y) * w) + (px + x);
								//			        			eprintln!(" {} {} -> {}", x, y, o);
								self.buffer[o] = pixel;
							}
						}
					} else {
						println!(
							"Error: No image for rendering {} ({}, {}) -> {}",
							i, x, y, &name
						);
					}
				},
				ButtonContent::Rgb(r, g, b) => {
					let x = i.wrapping_rem(self.width as usize);
					let y = (i / self.width as usize); //.floor();
								   //        			println!("Rendering {} ({}, {}) -> {}", i, x, y, &name);

					let px = BUTTON_SIZE * x;
					let py = BUTTON_SIZE * y;
					let w = BUTTON_SIZE * self.width;
					//        			let h = BUTTON_SIZE * self.height;
					let a = 0xff;
					let pixel: u32 = (((a & 0xff) as u32) << 24)
						| ((*r as u32) << 16) | ((*g as u32) << 8)
						| ((*b as u32) << 0);

					for y in 0..BUTTON_SIZE {
						for x in 0..BUTTON_SIZE {
							let o = ((py + y) * w) + (px + x);
							self.buffer[o] = pixel;
						}
					}
				},
				_ => {},
			}
		}

		if let Some(window) = &mut self.window {
			let mut old_buttons = self.buttons.clone();
			for b in self.buttons.iter_mut() {
				*b = 0;
			}
			self.button_changed = false;

			if let Some((x, y)) = window.get_mouse_pos(MouseMode::Discard) {
				if window.get_mouse_down(MouseButton::Left) {
					let x = x as usize / BUTTON_SIZE;
					let y = y as usize / BUTTON_SIZE;

					//	            	println!("Button {}, {} pressed", x, y );
					self.buttons[y * self.width + x] = 1;
				} else {
					//	            	println!("No button pressed");
				}
			}

			self.button_changed = old_buttons
				.iter()
				.zip(self.buttons.iter())
				.any(|(o, n)| o != n);

			self.done = window.is_open() && window.is_key_down(Key::Escape);
			window.update_with_buffer(&self.buffer, pw, ph).unwrap();
		}

		Ok(())
	}

	fn done(&self) -> bool {
		self.done
	}

	fn set_button_file(&mut self, index: u8, filename: &str) -> anyhow::Result<()> {
		if self.images.get(filename).is_none() {
			println!("First use of {}. Loading....", &filename);
			match image::open(&filename) {
				Ok(img) => {
					if img.dimensions() != (BUTTON_SIZE as u32, BUTTON_SIZE as u32) {
						println!(
							"Wrong dimensions for {} -> {}x{} need {}x{}",
							&filename,
							img.dimensions().0,
							img.dimensions().1,
							BUTTON_SIZE,
							BUTTON_SIZE
						);
						return Err(anyhow::anyhow!("Wrong image dimensions for {}", filename));
					} else {
						self.images.insert(filename.to_string(), img);
					}
					//				    self.images.push( img );
					//				    true
				},
				Err(e) => {
					println!("Couldn't load image {} {:?}", &filename, e);
					//				self.images = Vec::new();
					//					false
				},
			}
		}

		let index = index as usize;
		if let Some(bc) = &self.button_contents.get(index) {
			match bc {
				ButtonContent::Image { name } => {
					if name == filename {
						return Ok(());
					}
				},
				_ => {},
			}
		}

		self.button_contents[index] = ButtonContent::Image {
			name: filename.to_string(),
		};

		Ok(())
	}

	fn set_button_rgb(&mut self, index: u8, r: u8, g: u8, b: u8) -> anyhow::Result<()> {
		self.button_contents[index as usize] = ButtonContent::Rgb(r, g, b);
		Ok(())
	}

	fn read_buttons(&mut self, _timeout: Option<std::time::Duration>) -> anyhow::Result<Vec<u8>> {
		if self.button_changed {
			Ok(self.buttons.clone())
		} else {
			Err(anyhow::anyhow!("No buttons changed."))
		}
	}
}
