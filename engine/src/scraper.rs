use common::{
    define_scrapers,
    macros::{async_trait, paste},
};
use scraper::site::ExampleSite;

define_scrapers! {
    Example => ExampleSite,
}
