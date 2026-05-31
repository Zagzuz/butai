use std::{path::PathBuf, sync::Arc};

use chromiumoxide::{Browser, BrowserConfig, error::CdpError};
use clap::Parser;
use common::scraper::SharedBrowser;
use futures::StreamExt;
use thiserror::Error;
use tokio::{
    select,
    task::{JoinError, JoinHandle},
};
use tokio_util::sync::CancellationToken;
use tracing::error;

#[derive(Parser, Clone, Debug)]
pub struct BrowserOpts {
    /// Path to chromium executable
    #[clap(long, env = "BUTAI_CHROME_PATH")]
    chrome_path: Option<PathBuf>,
}

////////////////////////////////////////////////////////////////////////////////

pub struct BrowserHandle {
    token: CancellationToken,
    browser: SharedBrowser,
    handle: JoinHandle<()>,
}

impl BrowserHandle {
    pub async fn new(opts: BrowserOpts) -> Result<Self, BrowserError> {
        let token = CancellationToken::new();

        let mut builder = BrowserConfig::builder().new_headless_mode();
        if let Some(path) = opts.chrome_path {
            builder = builder.chrome_executable(path);
        }
        let config = builder.build().map_err(BrowserError::Build)?;
        let (browser, mut handler) = Browser::launch(config)
            .await
            .map_err(BrowserError::Launch)?;

        let t = token.clone();
        let handle = tokio::spawn(async move {
            loop {
                select! {
                    biased;
                    _ = t.cancelled() => {
                        break;
                    }
                    result = handler.next() => {
                        match result {
                            Some(_event) => {},
                            None => {
                                error!("Browser event stream is exhausted!");
                                break;
                            }
                        }
                    }
                }
            }
        });

        Ok(Self {
            token,
            browser: Arc::new(browser),
            handle,
        })
    }

    #[inline(always)]
    pub fn browser(&self) -> SharedBrowser {
        self.browser.clone()
    }

    pub async fn watch(&mut self) -> Result<(), JoinError> {
        (&mut self.handle).await
    }

    pub async fn cancel(mut self) {
        if let Some(browser) = Arc::get_mut(&mut self.browser) {
            if let Err(err) = browser.close().await {
                error!(%err, "Failed to close browser while cancelling");
            } else {
                match browser.wait().await {
                    Ok(_status) => {}
                    Err(err) => error!(%err, "Failed to wait for browser exit while cancelling"),
                };
            }
        }

        self.token.cancel();

        if !self.handle.is_finished() {
            if let Err(err) = self.handle.await {
                error!(?err, "Browser handler task finished with error while cancelling");
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Error)]
pub enum BrowserError {
    #[error("Browser failed to build, err = {0}")]
    Build(String),
    #[error("Browser failed to launch, err = {0}")]
    Launch(CdpError),
}
