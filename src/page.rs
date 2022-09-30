use crate::config::PageConfig;

#[derive(Debug, Default)]
pub struct Page {
	name:    String,
	buttons: Vec<Option<String>>,
}

impl Page {
	pub fn from_config(config: &PageConfig) -> anyhow::Result<Page> {
		let mut p = Page::default();
		p.name = config.name.clone();
		for bn in &config.buttons {
			if bn == "None" {
				p.buttons.push(None);
			} else {
				p.buttons.push(Some(bn.clone()))
			};
		}

		Ok(p)
	}

	pub fn name(&self) -> &str {
		&self.name
	}
	pub fn buttons(&self) -> &Vec<Option<String>> {
		&self.buttons
	}
}
