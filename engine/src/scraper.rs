use common::{
    define_scrapers,
    macros::{Url, async_trait, paste},
};
use scraper::site::ExampleSite;

define_scrapers! {
    ExampleSite => "https://example.com",
}
