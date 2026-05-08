use std::num::NonZeroUsize;
use std::time::Duration;
use clap::Parser;

#[derive(Parser, Debug)]
pub(super) struct Opts {
    /// Number of workers
    #[arg(short, long, env = "BUTAI_ENGINE_WORKERS")]
    pub(super) workers: Option<NonZeroUsize>,
    /// Scraping interval
    #[arg(short, long, default_value = "1h", value_parser = parse_interval)]
    pub(super) interval: Duration,
}

fn parse_interval(s: &str) -> Result<Duration, String> {
    let duration: Duration = s.parse::<humantime::Duration>()
        .map_err(|e| e.to_string())?
        .into();

    if duration < Duration::from_secs(60) {
        return Err("interval must be at least 1m".to_string());
    }

    Ok(duration)
}