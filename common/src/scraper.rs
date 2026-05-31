use std::sync::Arc;

use async_trait::async_trait;
use chromiumoxide::Browser;

pub type SharedBrowser = Arc<Browser>;

#[derive(Debug, Copy, Clone)]
pub enum ScrapeResult {
    Ok,
    Unsupported,
    Error,
}

impl ScrapeResult {
    pub fn unsupported() -> Self {
        Self::Unsupported
    }
}

#[async_trait]
pub trait HtmlScraper {
    async fn scrape_html(&self, _: SharedBrowser) -> ScrapeResult {
        ScrapeResult::unsupported()
    }
}

#[async_trait]
pub trait JsScraper {
    async fn scrape_js(&self) -> ScrapeResult {
        ScrapeResult::unsupported()
    }
}

#[async_trait]
pub trait ApiScraper {
    async fn scrape_api(&self) -> ScrapeResult {
        ScrapeResult::unsupported()
    }
}
