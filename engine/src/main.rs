use clap::Parser;

use crate::{engine::Engine, opts::Opts};

mod consts;
mod engine;
mod error;
mod opts;
mod utils;
mod scraper;
mod site;
mod limiter;
mod config;

fn main() {
    let opts = Opts::parse();
    let mut builder = tokio::runtime::Builder::new_multi_thread();
    if let Some(workers) = opts.workers {
        builder.worker_threads(workers.get());
    }
    builder
        .enable_io()
        .build()
        .expect("runtime failure")
        .block_on(Engine::start(opts))
        .expect("engine failure");
}
