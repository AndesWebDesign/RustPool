use std::io::Write;

use chrono::Local;
use log::{info, LevelFilter};

use crate::config::magic::RUSTPOOL_LOG_TIME_FORMAT_STRING;
use crate::constants::strings::{LOG_LEVEL_DEBUG,
                                LOG_LEVEL_ERROR,
                                LOG_LEVEL_INFO,
                                LOG_LEVEL_OFF,
                                LOG_LEVEL_TRACE,
                                LOG_LEVEL_WARNING,
                                LOG_STYLE_RUSTPOOL,
                                LOG_STYLE_SYSTEMD};
use crate::structs::Config;

fn get_level_filter(config: &Config) -> LevelFilter {
    match config.log_level.as_str() {
        LOG_LEVEL_OFF => {
            return LevelFilter::Off;
        }
        LOG_LEVEL_ERROR => {
            return LevelFilter::Error;
        }
        LOG_LEVEL_WARNING => {
            return LevelFilter::Warn;
        }
        LOG_LEVEL_INFO => {
            return LevelFilter::Info;
        }
        LOG_LEVEL_DEBUG => {
            return LevelFilter::Debug;
        }
        LOG_LEVEL_TRACE => {
            return LevelFilter::Trace;
        }
        _ => {
            panic!("could not parse log level: {}", config.log_level)
        }
    }
}

pub fn init_logging(config: &Config) {
    match &config.log_style as &str {
        LOG_STYLE_SYSTEMD => {
            env_logger::builder().format(|buf, record| {
                writeln!(
                    buf,
                    "<{}>{}: {}",
                    match record.level() {
                        log::Level::Error => 3,
                        log::Level::Warn => 4,
                        log::Level::Info => 6,
                        log::Level::Debug => 7,
                        log::Level::Trace => 7,
                    },
                    record.target(),
                    record.args()
                )
            }).filter_level(get_level_filter(config)).init()
        }
        LOG_STYLE_RUSTPOOL => {
            env_logger::builder().format(|buf, record| {
                writeln!(
                    buf,
                    "[ {} ] {} - {} - {}:{} - {}",
                    Local::now().format(RUSTPOOL_LOG_TIME_FORMAT_STRING),
                    record.level().as_str().to_uppercase(),
                    record.target(),
                    record.file().unwrap_or("unknown"),
                    record.line().unwrap_or(0),
                    record.args()
                )
            }).filter_level(get_level_filter(config)).init()
        }
        _ => {
            info!("could not parse log style: {}, using default", config.log_style);
            env_logger::builder().filter_level(get_level_filter(config)).init()
        }
    };
}
