
use lazy_static::lazy_static;
use regex::Regex;

#[derive(Debug)]
pub enum Action {
	None,
	Clear( u8, u8, u8 ),			// will clear all images, but this will be overwritten next frame
	Shutdown,
}

impl Default for Action {
	fn default() -> Action {
		Action::None
	}
}

impl From<&str> for Action {
	fn from( v: &str ) -> Self {
		match v {
			"Clear"		=> Action::Clear( 0, 0, 0 ),
			"Shutdown"	=> Action::Shutdown,
			_ => {
				lazy_static! {
     			   static ref RE: Regex = Regex::new(r"Clear\(\s*(\d+)\s*,\s*(\d+)\s*,\s*(\d+)\s*\)").unwrap();
     			}
//				let re = Regex::new(r"Clear\(\s*(\d+)\s*,\s*(\d+)\s*,\s*(\d+)\s*\)").unwrap_or( Regex::default());

				if let Some( c ) = RE.captures( v ) {
					dbg!(&c);
					if c.len() == 4 {
						Action::Clear(
							u8::from_str_radix( &c[1], 10 ).unwrap_or( 200 ),
							u8::from_str_radix( &c[2], 10 ).unwrap_or(  50 ),
							u8::from_str_radix( &c[3], 10 ).unwrap_or( 200 ),
						)
					} else {
						Action::Clear( 200, 50, 200 )
					}
				} else {
					Action::None
				}
			},
		}
	}
}


impl From<&String> for Action {
	fn from( v: &String ) -> Self {
		let v = v.as_str();
		v.into()
	}
}

