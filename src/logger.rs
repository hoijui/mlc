/*
 * SPDX-FileCopyrightText: 2019 - 2022 Armin Becher <becherarmin@gmail.com>
 * SPDX-FileCopyrightText: 2022 - 2025 Robin Vobruba <hoijui.quaero@gmail.com>
 *
 * SPDX-License-Identifier: MIT
 */

use log::debug;
use serde::Deserialize;
use simplelog::{ColorChoice, CombinedLogger, Config, LevelFilter, TermLogger, TerminalMode};
use std::time::SystemTime;

#[derive(Debug, Default, Clone, Copy, Deserialize)]
pub enum LogLevel {
    Info,
    #[default]
    Warn,
    Debug,
}

// pub fn init(log_level: LogLevel) {
//     let level_filter = match log_level {
//         LogLevel::Info => LevelFilter::Info,
//         LogLevel::Warn => LevelFilter::Warn,
//         LogLevel::Debug => LevelFilter::Debug,
//     };

//     let err = CombinedLogger::init(vec![TermLogger::new(
//         level_filter,
//         Config::default(),
//         TerminalMode::Stderr,
//         ColorChoice::Auto,
//     )]);
//     assert!(err.is_ok(), "Failed to init logger! Error: {err:?}");
//     debug!("Initialized logging");
// }

pub fn init(log_level: log::LevelFilter) -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "\x1B[{}m[{} {} {}] {}\x1B[0m",
                match record.level() {
                    log::Level::Error => "31", // Red
                    log::Level::Warn => "33",  // Yellow
                    log::Level::Info => "32",  // Green
                    log::Level::Debug => "34", // Blue
                    log::Level::Trace => "37", // White
                },
                SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                record.level(),
                record.target(),
                message
            ));
        })
        .level(log_level)
        .chain(std::io::stdout())
        .apply()?;
    debug!("Initialized logging");
    Ok(())
}
