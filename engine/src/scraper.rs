use common::{
    define_scrapers,
    macros::{Url, paste},
};
use scraper::site::OrixTheater;

define_scrapers! {
    OrixTheater => "https://orixtheater.jp",
}
