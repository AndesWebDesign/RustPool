extern crate clap;
extern crate env_logger;
extern crate hex;
extern crate log;
extern crate monero;
extern crate num_cpus;
extern crate randomx_rs;

use clap::{App, Arg};

use crate::config::loader::init_config;
use crate::logging::init_logging;
use crate::pool::exec::run_pool;

mod pool;
mod logging;
mod stream;
mod data;
mod config;
mod algo;
mod constants;
mod structs;
mod util;

fn main() {
    let matches = App::new("RustPool")
        .version("0.1.0")
        .author("erdos4d")
        .about("A Monero mining pool server written in Rust.")
        .arg(Arg::with_name("config_file")
            .short("c")
            .long("config_file")
            .help("The config file absolute path. Expects .json, .yaml, or .toml file.")
            .takes_value(true))
        .get_matches();
    let config_file_path = matches.value_of("config_file").unwrap_or("");
    let config = init_config(config_file_path);
    init_logging(&config);
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(config.threads as usize)
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            run_pool(&config).await;
        })
}
