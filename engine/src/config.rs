use serde::{Deserialize, Deserializer, Serialize};
use slotmap::{SecondaryMap, SlotMap, new_key_type};
use tokio::sync::RwLock;
use url::Url;

use crate::site::SiteHints;

new_key_type! {
    pub struct SiteKey;
}

pub struct Config {
    pub sites: SlotMap<SiteKey, Url>,
    pub hints: RwLock<SecondaryMap<SiteKey, SiteHints>>,
}

#[derive(Deserialize, Serialize)]
struct RawSite {
    url: Url,
    #[serde(default)]
    hints: SiteHints,
}

impl Config {
    pub async fn snapshot(&self) -> toml::Table {
        let hints = self.hints.read().await;
        let sites: toml::Value = self
            .sites
            .iter()
            .map(|(key, url)| {
                toml::Value::try_from(RawSite {
                    url: url.clone(),
                    hints: hints[key],
                })
                .unwrap()
            })
            .collect::<Vec<_>>()
            .into();

        toml::map::Map::from_iter([("sites".to_string(), sites)])
    }
}

impl<'de> Deserialize<'de> for Config {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct RawConfig {
            sites: Vec<RawSite>,
        }

        #[derive(Deserialize)]
        struct RawSite {
            url: Url,
            #[serde(default)]
            hints: SiteHints,
        }

        let raw = RawConfig::deserialize(d)?;
        let mut sites = SlotMap::<SiteKey, _>::default();
        let mut hints = SecondaryMap::<SiteKey, _>::new();

        for site in raw.sites {
            let key = sites.insert(site.url);
            hints.insert(key, site.hints);
        }

        Ok(Config {
            sites,
            hints: RwLock::new(hints),
        })
    }
}
