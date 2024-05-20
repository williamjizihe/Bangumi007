use std::sync::RwLock;
use log4rs;
use log4rs::append::console::ConsoleAppender;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Logger};
use log4rs::encode::pattern::PatternEncoder;
use crate::config::CONFIG;
use lazy_static::lazy_static;
use log4rs::{Config, Handle};

lazy_static! {
    pub static ref LOG_HANDLE: RwLock<Handle> = RwLock::new(init_logging());
}

pub fn get_logging_config() -> Config {
    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d(%H:%M:%S)} {h({l}):<5.5} [{M}] {m}{n}")))
        .build();

    let file = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d(%Y-%m-%d %H:%M:%S%.6f)} {h({l}):<5.5} [{M}] {m}{n}")))
        .build("log/bangumi007.log")
        .unwrap();

    fn get_log_level() -> log::LevelFilter {
        match CONFIG.read().unwrap().log_config.log_level.as_str() {
            "trace" => log::LevelFilter::Trace,
            "debug" => log::LevelFilter::Debug,
            "info" => log::LevelFilter::Info,
            "warn" => log::LevelFilter::Warn,
            "error" => log::LevelFilter::Error,
            _ => {
                log::warn!("Invalid log level, using warn instead");
                log::LevelFilter::Warn
            },
        }
    }

    // output to stdout and file
    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .appender(Appender::builder().build("file", Box::new(file)))
        .logger(Logger::builder().appender("file").build("bangumi007", get_log_level()))
        .build(
            log4rs::config::Root::builder()
                .appender("stdout")
                .appender("file")
                .build(get_log_level())
        )
        .unwrap();
    config
}

fn init_logging() -> Handle {
    let handle = log4rs::init_config(get_logging_config()).unwrap();
    log::debug!("Log level: {:?}", CONFIG.read().unwrap().log_config.log_level);
    handle
}

// Should be called at the beginning of the program
pub fn init() {
    let _a = LOG_HANDLE.read().unwrap();
}