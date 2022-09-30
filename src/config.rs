use regex::Regex;
use serde::Deserialize;
use serde_yaml;

#[derive(Debug, Default, Deserialize)]
pub struct EventConfig {
	pub http_request: Option<String>,
	pub actions:      Option<Vec<String>>,
}

#[derive(Debug, Default, Deserialize)]
pub struct ButtonConfig {
	pub name:         String,
	#[serde(rename = "type")]
	pub ttype:        String,
	pub image:        Option<String>,
	pub image_active: Option<String>,
	pub on_press:     Option<EventConfig>,
	pub on_release:   Option<EventConfig>,
}

#[derive(Debug, Default, Deserialize)]
pub struct PageConfig {
	pub name:    String,
	pub buttons: Vec<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct Config {
	buttons: Vec<ButtonConfig>,
	pages:   Vec<PageConfig>,
	/*
		#[serde(skip)]
		author_config: AuthorConfig,
		#[serde(skip)]
		tag_config: TagConfig,
	*/
}

impl Config {
	pub fn from_file(filename: &str) -> anyhow::Result<Config> {
		let f = std::fs::File::open(&filename)?;
		let mut c: Config = serde_yaml::from_reader(&f)?;
		c.expand()?;
		Ok(c)
	}

	pub fn buttons(&self) -> &Vec<ButtonConfig> {
		&self.buttons
	}
	pub fn pages(&self) -> &Vec<PageConfig> {
		&self.pages
	}

	pub fn expand(&mut self) -> anyhow::Result<()> {
		for b in self.buttons.iter_mut() {
			b.expand()?;
		}
		for p in self.pages.iter_mut() {
			p.expand()?;
		}
		Ok(())
	}
}

impl PageConfig {
	pub fn expand(&mut self) -> anyhow::Result<()> {
		Ok(())
	}
}
impl ButtonConfig {
	pub fn expand(&mut self) -> anyhow::Result<()> {
		if let Some(ec) = &mut self.on_press {
			ec.expand()?;
		}
		if let Some(ec) = &mut self.on_release {
			ec.expand()?;
		}
		Ok(())
	}
}
impl EventConfig {
	pub fn expand(&mut self) -> anyhow::Result<()> {
		let re_env = Regex::new(r"^(.*)(\$\{env:(.+)\})(.*)$").unwrap();
		if let Some(actions) = &mut self.actions {
			for a in actions.iter_mut() {
				let old_a = a.clone();
				let mut num_of_expansions = 0;
				let mut any_expansion = true;
				while any_expansion {
					any_expansion = false;
					if let Some(mat) = re_env.captures(a) {
						//println!("{:?}", &mat);
						let prefix = &mat[1];
						let varname = &mat[3];
						let suffix = &mat[4];
						let varvalue = std::env::var(varname)
							.expect(&format!("Env var {} not found", &varname));

						let new_a = prefix.to_string() + &varvalue + suffix;
						//println!("{} -> {}", &a, &new_a);
						*a = new_a;
						num_of_expansions += 1;
						any_expansion = true;
					}
				}
				if num_of_expansions > 0 {
					println!("{}\n\t-> {}", &old_a, &a);
				}
			}
		}
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use crate::Config;

	#[test]
	fn config_from_file() {
		let c = Config::from_file(&"config.yaml");

		dbg!(&c);
	}
}
