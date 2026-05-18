pub use async_trait::async_trait;
pub use paste::paste;

#[macro_export]
macro_rules! define_scrapers {
    ($($name:ident => $ty:ty),* $(,)?) => {
        paste! {
            pub enum AnyScraper {
                $($name($ty),)*
            }

            #[async_trait]
            impl $crate::scraper::HtmlScraper for AnyScraper {
                async fn scrape_html(&self) -> $crate::scraper::ScrapeResult {
                    match self {
                        $(Self::$name(s) => s.scrape_html().await,)*
                    }
                }
            }

            #[async_trait]
            impl $crate::scraper::JsScraper for AnyScraper {
                async fn scrape_js(&self) -> $crate::scraper::ScrapeResult {
                    match self {
                        $(Self::$name(s) => s.scrape_js().await,)*
                    }
                }
            }

            #[async_trait]
            impl $crate::scraper::ApiScraper for AnyScraper {
                async fn scrape_api(&self) -> $crate::scraper::ScrapeResult {
                    match self {
                        $(Self::$name(s) => s.scrape_api().await,)*
                    }
                }
            }
        }
    }
}
