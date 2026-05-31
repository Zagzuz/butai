use std::io;

use thiserror::Error;

use crate::browser::BrowserError;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to read configuration, {0}")]
    ReadConfig(io::Error),
    #[error("Failed to parse configuration, {0}")]
    InvalidConfig(toml::de::Error),
    #[error(transparent)]
    Browser(#[from] BrowserError),
}
