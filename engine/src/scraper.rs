use enum_dispatch::enum_dispatch;

pub type ScrapeResult = ();

#[enum_dispatch]
pub enum ScraperKind {
    /// API
    GhostScraper,
    /// Javascript
    ScoutScraper,
    /// HTML
    ScriptScraper,
}

#[enum_dispatch(ScraperKind)]
pub trait Scraper {
    async fn scrape(&self) -> ScrapeResult;
}

////////////////////////////////////////////////////////////////////////////////

pub struct ScoutScraper;

impl Scraper for ScoutScraper {
    async fn scrape(&self) -> ScrapeResult {
        todo!()
    }
}

pub struct GhostScraper;

impl Scraper for GhostScraper {
    async fn scrape(&self) -> ScrapeResult {
        todo!()
    }
}

pub struct ScriptScraper;

impl Scraper for ScriptScraper {
    async fn scrape(&self) -> ScrapeResult {
        todo!()
    }
}
