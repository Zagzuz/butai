pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub const DEFAULT_LOG_FILTERS: &[&str] = &[
    "mio=info",
    "hyper::proto=info",
    "hyper::client=info",
    "hyper_util::client=info",
    "reqwest::connect=info",
    "tonic::codec=info",
    "tower::buffer=info",
    "h2::codec=info",
    "h2::client=info",
    "h2::proto=info",
];
