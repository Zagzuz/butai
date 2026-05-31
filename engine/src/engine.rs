use std::{sync::Arc, time::Duration};

use common::scraper::{ApiScraper, HtmlScraper, JsScraper, ScrapeResult};
use futures::{StreamExt, stream::FuturesUnordered};
use humantime::format_duration;
use tokio::{
    select,
    time::{MissedTickBehavior, interval},
};
use tracing::{debug, error, info, warn};

use crate::{
    config::{Config, SiteKey},
    consts::VERSION,
    error::Error,
    opts::Opts,
    scraper::{self, ScraperMap},
    site::ScraperKind,
    utils,
};

// TODO: move to opts
const CONFIG_PATH: &'static str = "butai.toml";

pub(super) struct Engine;

impl Engine {
    pub(super) async fn start(opts: Opts) -> Result<(), Error> {
        info!("Starting Butai Engine v{VERSION}...");

        info!("Loading configuration...");
        let config: Arc<Config> = Arc::new(
            toml::from_slice(
                &tokio::fs::read(CONFIG_PATH)
                    .await
                    .map_err(Error::ReadConfig)?,
            )
            .map_err(Error::InvalidConfig)?,
        );
        info!("Configuration loaded, {} websites in total", config.sites.len());

        let scrapers = Arc::new(scraper::build_scrapers());

        // TODO: serve metrics

        loop {
            info!("Butai Engine started");

            select! {
                biased;
                _ = utils::shutdown_signal() => {
                    info!("Engine got a shutdown signal");
                    break Ok(());
                }
                ret = Self::analyze(opts.interval, config.clone(), scrapers.clone()) => {
                    error!(?ret, "Engine analyzer finished unexpectedly");
                    break Ok(());
                }
            }
        }
    }

    async fn analyze(scrape_interval: Duration, config: Arc<Config>, scrapers: Arc<ScraperMap>) {
        let mut i = interval(scrape_interval);
        i.set_missed_tick_behavior(MissedTickBehavior::Delay);

        loop {
            i.tick().await;
            info!("Scraping initiated");

            let mut workers = config
                .sites
                .keys()
                .map(|token| {
                    Worker {
                        key: token,
                        config: config.clone(),
                        scrapers: scrapers.clone(),
                    }
                    .run()
                })
                .collect::<FuturesUnordered<_>>();

            loop {
                match workers.next().await {
                    Some(()) => {}
                    None => break,
                }
            }

            info!(
                "Scraping finished, sleeping for {}...",
                format_duration(scrape_interval)
            );
        }
    }
}

struct Worker {
    key: SiteKey,
    config: Arc<Config>,
    scrapers: Arc<ScraperMap>,
}

impl Worker {
    async fn run(self) {
        // TODO: Deescalate after 1000 stable scrapes

        let url = &self.config.sites[self.key];
        let Some(scraper) = self.scrapers.get(url) else {
            // TODO: Universal scraper fallback
            warn!(%url, "Unsupported website, skipping...");
            return;
        };
        let mut hints = self.config.hints.read().await[self.key];
        let mut kind = hints.scraper.unwrap_or(ScraperKind::Ghost);

        loop {
            debug!(%url, ?kind, ?hints, "Attempting scraping...");
            let scrape_result = match kind {
                ScraperKind::Ghost => scraper.scrape_api(url).await,
                ScraperKind::Script => scraper.scrape_html(url).await,
                ScraperKind::Scout => scraper.scrape_js(url).await,
            };

            match scrape_result {
                ScrapeResult::Ok => {
                    hints.scraper.replace(kind);
                    hints.stable_count += 1;
                    debug!(%url, ?kind, "Scraping succeeded");
                    break;
                }
                ScrapeResult::Unsupported => {
                    if !kind.elevate() {
                        warn!(%url, "No scraper implementations found, skipping...");
                        hints.stable_count = 0;
                        // TODO: Universal scraper fallback
                        break;
                    }
                }
                ScrapeResult::Error => {
                    error!(%url, ?kind, "Scraper failed");
                    hints.stable_count = 0;
                    if !kind.elevate() {
                        error!(%url, "All scrapers failed, skipping...");
                        // TODO: Universal scraper fallback
                        break;
                    }
                }
            }
        }

        self.config.hints.write().await.insert(self.key, hints);
        // TODO: persist config
        /*if let Err(err) = tokio::fs::write(CONFIG_PATH, self.config.snapshot()).await {
            error!(?err, "Config update failed!");
        }*/
    }
}
