pub use async_trait::async_trait;
pub use paste::paste;
pub use url::Url;

#[cfg(feature = "scraper")]
#[macro_export]
macro_rules! define_scrapers {
    ($($ty:ty => $url:literal),* $(,)?) => {
        pub type ScraperMap = std::collections::HashMap<Url, AnyScraper>;

        paste! {
            $(
                static [<$ty:snake:upper _URL>]: std::sync::LazyLock<Url> =
                    std::sync::LazyLock::new(|| Url::parse($url).expect("invalid scraper url"));
            )*

            pub enum AnyScraper {
                $([<$ty>]($ty),)*
            }

            #[async_trait]
            impl $crate::scraper::HtmlScraper for AnyScraper {
                async fn scrape_html(&self, url: &Url) -> $crate::scraper::ScrapeResult
                {
                    match self {
                        $(Self::[<$ty>](s) if url == &*[<$ty:snake:upper _URL>] => {
                            s.scrape_html(url).await
                        },)*
                        _ => $crate::scraper::ScrapeResult::unsupported(),
                    }
                }
            }

            #[async_trait]
            impl $crate::scraper::JsScraper for AnyScraper {
                async fn scrape_js(&self, url: &Url) -> $crate::scraper::ScrapeResult
                {
                    match self {
                        $(Self::[<$ty>](s) if url == &*[<$ty:snake:upper _URL>] => {
                            s.scrape_js(url).await
                        },)*
                        _ => $crate::scraper::ScrapeResult::unsupported(),
                    }
                }
            }

            #[async_trait]
            impl $crate::scraper::ApiScraper for AnyScraper {
                async fn scrape_api(&self, url: &Url) -> $crate::scraper::ScrapeResult
                {
                    match self {
                        $(Self::[<$ty>](s) if url == &*[<$ty:snake:upper _URL>] => {
                            s.scrape_api(url).await
                        },)*
                        _ => $crate::scraper::ScrapeResult::unsupported(),
                    }
                }
            }

            pub fn build_scrapers() -> ScraperMap {
                [$(([<$ty:snake:upper _URL>].clone(), AnyScraper::[<$ty>]($ty)),)*].into()
            }
        }
    }
}
