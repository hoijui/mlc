/*
 * SPDX-FileCopyrightText: 2019 - 2022 Armin Becher <becherarmin@gmail.com>
 * SPDX-FileCopyrightText: 2022 - 2025 Robin Vobruba <hoijui.quaero@gmail.com>
 *
 * SPDX-License-Identifier: MIT
 */

use log::debug;
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
        TerminalMode::Stderr,
        ColorChoice::Auto,
    )]);
    assert!(err.is_ok(), "Failed to init logger! Error: {err:?}");
    debug!("Initialized logging");
}
