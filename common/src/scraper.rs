use async_trait::async_trait;
use url::Url;

#[derive(Debug, Copy, Clone)]
pub enum ScrapeResult {
    Unsupported,
}

impl ScrapeResult {
    pub fn unsupported() -> Self {
        Self::Unsupported
    }
}

#[async_trait]
pub trait HtmlScraper {
    async fn scrape_html(&self, _: &Url) -> ScrapeResult {
        ScrapeResult::unsupported()
    }
}

#[async_trait]
pub trait JsScraper {
    async fn scrape_js(&self, _: &Url) -> ScrapeResult {
        ScrapeResult::unsupported()
    }
}

#[async_trait]
pub trait ApiScraper {
    async fn scrape_api(&self, _: &Url) -> ScrapeResult {
        ScrapeResult::unsupported()
    }
}
