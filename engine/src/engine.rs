use std::{sync::Arc, time::Duration};

use chromiumoxide::Browser;
use common::scraper::{ScrapeResult, SharedBrowser};
use futures::{StreamExt, stream::FuturesUnordered};
use humantime::format_duration;
use tokio::{
    select,
    time::{MissedTickBehavior, interval},
};
use tracing::{debug, error, info, warn};

use crate::{
    browser::BrowserHandle,
    config::{Config, SiteKey},
    consts::VERSION,
    error::Error,
    opts::Opts,
    scraper::{self, ScraperMap},
    site::ScraperKind,
    utils,
};

// TODO: move to opts
const CONFIG_PATH: &str = "butai.toml";

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
        let mut browser_handle = BrowserHandle::new(opts.browser).await?;

        // TODO: serve metrics

        'l: {
            info!("Butai Engine started");

            select! {
                biased;
                _ = utils::shutdown_signal() => {
                    info!("Engine got a shutdown signal");
                    break 'l;
                }
                ret = Self::analyze(
                    opts.interval,
                    config.clone(),
                    scrapers.clone(),
                    browser_handle.browser()
                ) => {
                    error!(?ret, "Engine analyzer finished unexpectedly");
                    break 'l;
                }
                ret = browser_handle.watch() => {
                    match ret {
                        Ok(()) => error!("Browser event task finished unexpectedly"),
                        Err(err) => error!(?err, "Browser event task failed"),
                    }
                    // TODO: recovery + backoff
                    break 'l;
                }
            }
        }

        browser_handle.cancel().await;

        Ok(())
    }

    async fn analyze(
        scrape_interval: Duration,
        config: Arc<Config>,
        scrapers: Arc<ScraperMap>,
        browser: SharedBrowser,
    ) {
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
                        browser: browser.clone(),
                    }
                    .run()
                })
                .collect::<FuturesUnordered<_>>();

            while let Some(()) = workers.next().await {}

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
    browser: Arc<Browser>,
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
                ScraperKind::Script => scraper.scrape_html(url, self.browser.clone()).await,
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
        if let Err(err) =
            tokio::fs::write(CONFIG_PATH, self.config.snapshot().await.to_string()).await
        {
            error!(?err, "Config update failed!");
        }
    }
}
