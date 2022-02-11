
use crate::action::Action;
use crate::config::ButtonConfig;

#[derive(Debug)]
enum ButtonType {
	None,	
	Simple {
		image:			Option< String >,
		image_active:	Option< String >,
	},
}

impl Default for ButtonType {
	fn default() -> ButtonType {
		ButtonType::None
	}
}

#[derive(Debug, Default)]
pub struct Button {
	name:		String,
	ttype:		ButtonType,
	pressed:	bool,
	on_press:	Vec< Action >,
	on_release:	Vec< Action >,
}

impl Button {
	pub fn from_config( config: &ButtonConfig ) -> anyhow::Result<Button> {
		let mut b = match config.ttype.as_str() {
			"simple" => {
				let mut b = Button::default();
				b.ttype = ButtonType::Simple {
					image:			config.image.clone(),
					image_active:	config.image_active.clone(),
				};

				b
			},
			t => return Err( anyhow::anyhow!("Unsupported button type: {:?}", t ) ),
		};

		b.name = config.name.clone();

		if let Some( event_config ) = &config.on_release {
			if let Some( actions ) = &event_config.actions {
				for action in actions {
					b.on_release.push( action.into() );
				}
			} // else :TODO:
		}

		Ok( b )
	}

	pub fn update( &mut self ) {

	}

	pub fn press( &mut self ) -> &Vec< Action > {
		self.pressed = true;
		&self.on_press
	}

	pub fn release( &mut self ) -> &Vec< Action > {
		self.pressed = false;
		&self.on_release
	}

	pub fn image( &self ) -> Option< &str >{
		match &self.ttype {
			ButtonType::Simple{ image, image_active } => {
				if self.pressed {
					image_active.as_deref()
				} else {
					image.as_deref()
				}
			},
			_ => None,
		}
	}
}
