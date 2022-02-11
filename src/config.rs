use serde::Deserialize;
use serde_yaml;

#[derive(Debug, Default, Deserialize)]
pub struct EventConfig {
    pub http_request:   Option< String >,
    pub actions:         Option< Vec< String > >,
}

#[derive(Debug, Default, Deserialize)]
pub struct ButtonConfig {
    pub name:           String,
    #[serde(rename = "type")]
    pub ttype:          String,
    pub image:          Option< String >,
    pub image_active:   Option< String >,
    pub on_press:       Option< EventConfig >,
    pub on_release:     Option< EventConfig >,
}

#[derive(Debug, Default, Deserialize)]
pub struct PageConfig {
    pub name:           String,
    pub buttons:        Vec< String >,
}

#[derive(Debug, Default, Deserialize)]
pub struct Config {
    buttons:        Vec<ButtonConfig>,
    pages:          Vec<PageConfig>,

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
/*
        if let Some(f) = &c.author_config_file {
            c.author_config = AuthorConfig::from_file(&f)?;
        }

        if let Some(f) = &c.tag_config_file {
            c.tag_config = TagConfig::from_file(&f)?;
        }
*/
        Ok(c)
    }

/*
    pub fn author_config(&self) -> &AuthorConfig {
        &self.author_config
    }

    pub fn tag_config(&self) -> &TagConfig {
        &self.tag_config
    }
*/

    pub fn buttons( &self ) -> &Vec< ButtonConfig > {
        &self.buttons
    }
    pub fn pages( &self ) -> &Vec< PageConfig > {
        &self.pages
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
