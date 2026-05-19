use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ScraperKind {
    Ghost,
    Scout,
    Script,
}

impl ScraperKind {
    pub fn elevate(&mut self) -> bool {
        *self = match self {
            ScraperKind::Ghost => ScraperKind::Script,
            ScraperKind::Scout => return false,
            ScraperKind::Script => ScraperKind::Scout,
        };

        true
    }
}

#[derive(Debug, Default, Copy, Clone, Deserialize, Serialize)]
pub struct SiteHints {
    pub scraper: Option<ScraperKind>,
    #[serde(default)]
    pub stable_count: u32,
}

#[cfg(test)]
mod tests {
    use futures::executor::block_on;
    use serde::{Deserialize, de::IntoDeserializer};
    use toml::toml;
    use url::Url;

    use crate::config::Config;

    #[test]
    fn ser_de_config() {
        let url = Url::parse("https://google.com").unwrap().to_string();
        let table = toml! {
            [[sites]]
            url = url
            hints.scraper = "ghost"
            hints.stable_count = 1
        };
        let deserializer = table.clone().into_deserializer();
        let config = Config::deserialize(deserializer).unwrap();
        assert_eq!(
            block_on(config.snapshot()),
            table,
            "deserialized config snapshot != toml table"
        );
    }
}
