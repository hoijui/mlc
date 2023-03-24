extern crate simplelog;

use serde::Deserialize;
use simplelog::{ColorChoice, CombinedLogger, Config, LevelFilter, TermLogger, TerminalMode};

#[derive(Debug, Default, Clone, Copy, Deserialize)]
pub enum LogLevel {
    Info,
    #[default]
    Warn,
    Debug,
}

pub fn init(log_level: &LogLevel) {
    let level_filter = match log_level {
        LogLevel::Info => LevelFilter::Info,
        LogLevel::Warn => LevelFilter::Warn,
        LogLevel::Debug => LevelFilter::Debug,
    };

    let err = CombinedLogger::init(vec![TermLogger::new(
        level_filter,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )]);
    if err.is_err() {
        panic!("Failed to init logger! Error: {:?}", err);
    }
    debug!("Initialized logging");
}
