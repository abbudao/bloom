// Bloom
//
// HTTP REST API caching middleware
// Copyright: 2017, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

#[macro_use]
extern crate log;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;
extern crate brotli;
extern crate farmhash;
extern crate futures;
extern crate futures_cpupool;
extern crate httparse;
extern crate hyper;
extern crate r2d2;
extern crate r2d2_redis;
extern crate rand;
extern crate redis;
extern crate tokio_core;
extern crate toml;
extern crate unicase;

mod cache;
mod config;
mod control;
mod header;
mod proxy;
mod server;

use std::str::FromStr;
use std::thread;
use std::time::Duration;

use clap::{App, Arg};
use log::LevelFilter;

use cache::store::CacheStore;
use config::{Config, ConfigLogger, ConfigReader};
use control::listen::ControlListen;
use server::listen::ServerListen;

struct AppArgs {
    config: String,
}

pub static LINE_FEED: &str = "\r\n";

pub static THREAD_NAME_WORKER: &str = "bloom-worker";
pub static THREAD_NAME_CONTROL_MASTER: &str = "bloom-control-master";
pub static THREAD_NAME_CONTROL_CLIENT: &str = "bloom-control-client";

lazy_static! {
    static ref APP_ARGS: AppArgs = make_app_args();
    static ref APP_CONF: Config = ConfigReader::make();
    static ref APP_CACHE_STORE: CacheStore = CacheStore::new();
}

fn make_app_args() -> AppArgs {
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .help("Path to configuration file")
                .default_value("./config.cfg")
                .takes_value(true),
        )
        .get_matches();

    // Generate owned app arguments
    AppArgs {
        config: String::from(matches.value_of("config").expect("invalid config value")),
    }
}

fn ensure_states() {
    // Ensure all statics are valid (a `deref` is enough to lazily initialize them)
    let (_, _, _) = (&*APP_ARGS, &*APP_CONF, &*APP_CACHE_STORE);
}

fn spawn_worker() {
    let worker = thread::Builder::new()
        .name(THREAD_NAME_WORKER.to_string())
        .spawn(ServerListen::run);

    // Block on worker thread (join it)
    let has_error = if let Ok(worker_thread) = worker {
        worker_thread.join().is_err()
    } else {
        true
    };

    // Worker thread crashed?
    if has_error {
        error!("worker thread crashed, setting it up again");

        // Prevents thread start loop floods
        thread::sleep(Duration::from_secs(1));

        spawn_worker();
    }
}

fn main() {
    let _logger = ConfigLogger::init(
        LevelFilter::from_str(&APP_CONF.server.log_level).expect("invalid log level"),
    );

    info!("starting up");

    // Ensure all states are bound
    ensure_states();

    // Run control interface (in its own thread)
    ControlListen::run();

    // Run server (from main thread, maintain thread active if down)
    spawn_worker();

    error!("could not start");
}
