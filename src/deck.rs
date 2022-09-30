//use derivative::Derivative;

//const EMPTY: Vec<u8> = vec![];

//#[derive(Derivative,Default)]
//#[derivative(Debug)]
pub trait Deck {
	fn run(&mut self) -> anyhow::Result<()> {
		Ok(())
	}
	fn update(&mut self) -> anyhow::Result<()> {
		Ok(())
	}

	fn done(&self) -> bool {
		false
	}

	fn set_button_file(&mut self, _index: u8, _filename: &str) -> anyhow::Result<()> {
		Ok(())
	}

	fn set_button_rgb(&mut self, _index: u8, _r: u8, _g: u8, _b: u8) -> anyhow::Result<()> {
		Ok(())
	}

	fn read_buttons(&mut self, timeout: Option<std::time::Duration>) -> anyhow::Result<Vec<u8>>;
}

//impl dyn Deck {}
