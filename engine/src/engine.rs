use std::time::Duration;

use tokio::{
    select, spawn,
    time::{MissedTickBehavior, interval},
};
use tracing::{error, info};

use crate::{consts::VERSION, error::Error, opts::Opts, utils};

#[derive(Copy, Clone)]
pub(super) struct Engine {
    scrape_interval: Duration,
}

impl Engine {
    pub(super) async fn start(opts: Opts) -> Result<(), Error> {
        info!("Starting Butai Engine v{VERSION}...");

        let engine = Engine {
            scrape_interval: opts.interval,
        };

        loop {
            let handle = spawn(engine.enter());

            info!("Butai Engine started");

            select! {
                biased;
                _ = utils::shutdown_signal() => {
                    info!("Engine got a shutdown signal");
                    break;
                }
                ret = handle => {
                    match ret {
                        Ok(()) => {
                            error!("Butai Engine stopped unexpectedly, trying to restart...");
                            continue;
                        },
                        Err(err) => {
                            error!(?err, "Butai Engine failed on start");
                            break;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    async fn enter(self) {
        let mut i = interval(self.scrape_interval);
        i.set_missed_tick_behavior(MissedTickBehavior::Delay);

        loop {
            i.tick().await;

            // TODO: scraping impl
        }
    }
}
