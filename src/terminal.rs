use crossterm::{event, terminal};

#[derive(Debug, Default)]
pub struct Terminal {
	number_key_status: Vec<u8>,
}

impl Terminal {
	pub fn run(&mut self) -> anyhow::Result<()> {
		self.number_key_status.resize(10, 0);

		terminal::enable_raw_mode()?;

		tokio::spawn(async move {
			loop {
				match event::poll(std::time::Duration::from_millis(1_000)) {
					Ok(events_available) => {
						if events_available {
							match event::read() {
								Ok(event) => {
									eprintln!("{:?}", &event);
									match event {
										event::Event::Key(ke) => match (ke.code, ke.modifiers) {
											(
												event::KeyCode::Char('c'),
												event::KeyModifiers::CONTROL,
											) => {
												println!("Got ctrl-c. Exiting!");
												let _ = terminal::disable_raw_mode();
												std::process::exit(-1);
											},
											_ => {},
										},
										_ => {},
									}
								},
								Err(e) => {
									eprintln!("Error reading terminal: {:?}", &e);
								},
							}
						} else {
							eprintln!(".");
						}
					},
					Err(e) => {
						eprintln!("Error polling terminal: {:?}", &e);
					},
				}
			}
		});

		Ok(())
	}

	#[allow(dead_code)]
	pub fn number_key_status(&self) -> &Vec<u8> {
		&self.number_key_status
	}
}
