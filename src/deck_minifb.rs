
use minifb::{
	Key,
	Window,
	WindowOptions
};

use derivative::Derivative;

use crate::deck::Deck;

#[derive(Derivative,Default)]
#[derivative(Debug)]
pub struct Deck_Minifb {
	title:	String,
	width:	u8,		// in buttons
	height: u8,		// in buttons
}

impl Deck_Minifb {
	pub fn new( title: &str, w: u8, h: u8 ) -> Self {
		Self {
			title:		title.to_string(),
			width:		w,
			height:		h,
		}
	}

	pub fn run( &mut self ) -> anyhow::Result<()> {
		let button_size = 72;
		let pw = button_size * self.width as usize;
		let ph = button_size * self.height as usize;
		let title = self.title.clone();

		// window is not Send, so we have to initialise it inside the task :(
		let h : tokio::task::JoinHandle< Result<(), anyhow::Error> > = tokio::spawn(async move {

			let mut buffer: Vec<u32> = vec![0; pw * ph];

		    let mut window = Window::new(
		        &title,
		        pw,
		        ph,
		        WindowOptions::default(),
		    )?;
/*
			window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

			loop {
				window
		            .update_with_buffer(&buffer, pw, ph)
        		    .unwrap();
			};
*/
			loop {}
			Err(anyhow::anyhow!("Should not return"))
		});


		Ok(())
	}
}

impl Deck for Deck_Minifb {

}
