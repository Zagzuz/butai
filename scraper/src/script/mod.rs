use chrono::{Datelike, Local};
use common::{
    macros::async_trait,
    scraper::{HtmlScraper, ScrapeResult, SharedBrowser},
};
use tracing::trace;

use crate::site::OrixTheater;

#[async_trait]
impl HtmlScraper for OrixTheater {
    async fn scrape_html(&self, browser: SharedBrowser) -> ScrapeResult {
        let now = Local::now();
        let url = format!(
            "https://www.orixtheater.jp/event/?yearId={}&monthId={}",
            now.year(),
            now.month()
        );
        trace!(%url, "Scraping OrixTheater");
        let page = browser
            .new_page(url)
            .await
            .unwrap();
        let source = page
            .wait_for_navigation()
            .await
            .unwrap()
            .content()
            .await
            .unwrap();
        page.close().await.unwrap();

        println!("{source}");

        ScrapeResult::Ok
    }
}
