use std::{sync::Arc, time::Duration};

use futures::{StreamExt, stream::FuturesUnordered};
use tokio::{
    select,
    time::{MissedTickBehavior, interval},
};
use tracing::{error, info};

use crate::{
    config::{Config, SiteKey},
    consts::VERSION,
    error::Error,
    opts::Opts,
    site::SiteHint,
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

        loop {
            info!("Butai Engine started");

            select! {
                biased;
                _ = utils::shutdown_signal() => {
                    info!("Engine got a shutdown signal");
                    break Ok(());
                }
                ret = Self::analyze(opts.interval, config.clone()) => {
                    error!(?ret, "Engine analyzer finished unexpectedly");
                    break Ok(());
                }
            }
        }
    }

    async fn analyze(scrape_interval: Duration, config: Arc<Config>) {
        let mut i = interval(scrape_interval);
        i.set_missed_tick_behavior(MissedTickBehavior::Delay);

        loop {
            i.tick().await;

            let mut workers = config
                .sites
                .keys()
                .map(|token| {
                    Worker {
                        key: token,
                        config: config.clone(),
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
        }
    }
}

struct Worker {
    key: SiteKey,
    config: Arc<Config>,
}

impl Worker {
    async fn run(self) {
        //1. Surface analysis
        //    └─ check headers, meta tags, JS script tags in initial HTML
        //
        // 2. Assign initial scraper kind from hints
        //
        // 3. Scrape attempt
        //    └─ success → done
        //    └─ failure/incomplete → escalate kind, update hints, retry
        //
        // 4. Watcher picks up persistent failures
        //    └─ triggers re-analysis with heavier scraper if needed
        //
        // TODO: Deescalate after 1000 stable scrapes
        // TODO: Full hint reset after 10 consecutive failures
        // TODO: Unfeasible hints should be corrected by scrapers.

        self.update_hints().await;

        todo!("scrape")
    }

    async fn update_hints(&self) {
        let mut updated = false;
        let mut hints = self.config.hints.read().await[self.key];

        if hints.has_public_api.is_unknown() {
            hints.has_public_api = SiteHint::Detected(todo!("check"));
            updated = true;
        }

        if hints.has_hidden_api.is_unknown() {
            hints.has_hidden_api = SiteHint::Detected(todo!("check"));
            updated = true;
        }

        if hints.requires_auth.is_unknown() {
            hints.requires_auth = SiteHint::Detected(todo!("check"));
            updated = true;
        }

        if hints.has_js_rendering.is_unknown() {
            hints.has_js_rendering = SiteHint::Detected(todo!("check"));
            updated = true;
        }

        if updated {
            if self
                .config
                .hints
                .write()
                .await
                .insert(self.key, hints)
                .is_none()
            {
                error!("Hints update failed! Key was removed from the originating slot map.");
            }
            // TODO: persist config
        }
    }
}
