use clap::Parser;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{
    EnvFilter, filter::ParseError, fmt::layer, layer::SubscriberExt, registry,
    util::SubscriberInitExt,
};

use crate::consts::DEFAULT_LOG_FILTERS;

#[derive(Parser, Clone, Copy, Debug)]
pub struct LogOpts {
    #[clap(short, long, env = "BUTAI_LOG", default_value = "info")]
    log_level: LevelFilter,
}

impl LogOpts {
    pub fn init(&self) -> Result<(), ParseError> {
        let mut filter = EnvFilter::builder()
            .with_default_directive(self.log_level.into())
            .with_env_var("LOG_FILTERS")
            .from_env_lossy();

        for rule in DEFAULT_LOG_FILTERS {
            filter = filter.add_directive(rule.parse()?);
        }

        registry().with(filter).with(layer()).init();

        Ok(())
    }
}
