use lazy_static::lazy_static;
use regex::Regex;
//use regex::RegexSet;

#[derive(Debug, Clone)]
pub enum Action {
	None,
	Clear(u8, u8, u8), // will clear all images, but this will be overwritten next frame
	Shutdown,
	HttpGet(String),
	ObsScene(String, String),
	OscSend(String, String),
}

impl Default for Action {
	fn default() -> Action {
		Action::None
	}
}

fn clear_from_string(s: &str) -> Option<Action> {
	lazy_static! {
			static ref RE: Regex =
				Regex::new(
					r"^Clear\(\s*(\d+)\s*,\s*(\d+)\s*,\s*(\d+)\s*\)$"
	//								r"|^((HttpGet)\(\s*http:\\\\.+\s*\))$",
	//								r"|((Test))"
				).unwrap();
		}
	if let Some(c) = RE.captures(s) {
		dbg!(&c);
		println!("--------");
		if c.len() == 4 {
			return Some(Action::Clear(
				u8::from_str_radix(&c[1], 10).unwrap_or(200),
				u8::from_str_radix(&c[2], 10).unwrap_or(50),
				u8::from_str_radix(&c[3], 10).unwrap_or(200),
			));
		} else {
			return Some(Action::Clear(200, 50, 200));
		}
	}
	None
}

fn httpget_from_string(s: &str) -> Option<Action> {
	lazy_static! {
				static ref RE: Regex =
					Regex::new(
	//					r"^HttpGet\(\s*http:\\\\.*\s*\)$",
						r"^HttpGet\(\s*(http://.*)\s*\)$",
					).unwrap();
			}
	if let Some(c) = RE.captures(s) {
		dbg!(&c);
		println!("--------");
		if c.len() == 2 {
			return Some(Action::HttpGet(c[1].to_string()));
		}
	}
	None
}

fn obsscene_from_string(s: &str) -> Option<Action> {
	lazy_static! {
				static ref RE: Regex =
					Regex::new(
	//                  r"^HttpGet\(\s*http:\\\\.*\s*\)$",
						r"^ObsScene\(\s*(.+)\s*,\s*(.+)\s*\)$",
					).unwrap();
			}
	if let Some(c) = RE.captures(s) {
		dbg!(&c);
		println!("--------");
		if c.len() == 3 {
			return Some(Action::ObsScene(c[1].to_string(), c[2].to_string()));
		}
	}

	dbg!(s);
	None
}

fn oscsend_from_string(s: &str) -> Option<Action> {
	lazy_static! {
				static ref RE: Regex =
					Regex::new(
	//                  r"^HttpGet\(\s*http:\\\\.*\s*\)$",
						r"^OscSend\(\s*(.+)\s*,\s*(.+)\s*\)$",
					).unwrap();
			}
	if let Some(c) = RE.captures(s) {
		dbg!(&c);
		println!("--------");
		if c.len() == 3 {
			return Some(Action::OscSend(c[1].to_string(), c[2].to_string()));
		}
	}

	dbg!(s);
	None
}

impl From<&str> for Action {
	fn from(v: &str) -> Self {
		match v {
			"Clear" => Action::Clear(0, 0, 0),
			"Shutdown" => Action::Shutdown,
			_ => {
				if let Some(a) = clear_from_string(&v) {
					a
				} else if let Some(a) = httpget_from_string(&v) {
					a
				} else if let Some(a) = obsscene_from_string(&v) {
					a
				} else if let Some(a) = oscsend_from_string(&v) {
					a
				} else {
					Action::None
				}
			},
		}
	}
}

impl From<&String> for Action {
	fn from(v: &String) -> Self {
		let v = v.as_str();
		v.into()
	}
}
