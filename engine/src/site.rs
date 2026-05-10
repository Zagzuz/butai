use serde::{Deserialize, Deserializer};

#[derive(Default, Copy, Clone, Eq, PartialEq)]
pub enum SiteHint {
    #[default]
    Unknown,
    Detected(bool),
    Unfeasible,
}

impl SiteHint {
    pub fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown)
    }
}

impl<'de> Deserialize<'de> for SiteHint {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        match Option::<bool>::deserialize(d)? {
            None => Ok(SiteHint::Unknown),
            Some(b) => Ok(SiteHint::Detected(b)),
        }
    }
}

#[derive(Copy, Clone, Default, Deserialize, Eq, PartialEq)]
pub struct SiteHints {
    pub has_public_api: SiteHint,
    pub has_hidden_api: SiteHint,
    pub has_js_rendering: SiteHint,
    pub requires_auth: SiteHint,
}
