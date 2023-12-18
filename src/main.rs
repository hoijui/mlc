#[macro_use]
extern crate log;

use mlc::cli;
use mlc::logger;
use std::process;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = cli::parse_args();
    let log_level = match config.optional.debug {
        Some(true) => logger::LogLevel::Debug,
        _ => logger::LogLevel::Warn,
    };
    logger::init(&log_level);
    info!("Config: {}", &config);
    if mlc::run(&config).await.is_err() {
        process::exit(1);
    } else {
        process::exit(0);
    }
}
