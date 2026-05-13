use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SiteHint {
    #[default]
    Unknown,
    Found,
    #[serde(alias = "not found")]
    NotFound,
    Unfeasible,
}

impl SiteHint {
    pub fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown)
    }
}

#[derive(Debug, Copy, Clone, Default, Eq, PartialEq, Deserialize, Serialize)]
pub struct SiteHints {
    pub has_public_api: SiteHint,
    pub has_hidden_api: SiteHint,
    pub has_js_rendering: SiteHint,
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
            hints.has_public_api = "found"
            hints.has_hidden_api = "not_found"
            hints.has_js_rendering = "unfeasible"
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
