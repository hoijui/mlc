/*
 * SPDX-FileCopyrightText: 2019 - 2022 Armin Becher <becherarmin@gmail.com>
 * SPDX-FileCopyrightText: 2023 - 2025 Robin Vobruba <hoijui.quaero@gmail.com>
 *
 * SPDX-License-Identifier: MIT
 */

mod cli;
mod logger;

use clap::crate_version;
use log::info;
use mle::BoxResult;
use std::process;

fn print_header() {
    let width = 60;
    let header = format!("markup link checker - mlc v{:}", crate_version!());
    println!();
    println!("{:+<1$}", "", width);
    print!("+");
    print!("{: <1$}", "", width - 2);
    println!("+");
    print!("+");
    print!("{: ^1$}", header, width - 2);
    println!("+");
    print!("+");
    print!("{: <1$}", "", width - 2);
    println!("+");
    println!("{:+<1$}", "", width);
    println!();
}

#[tokio::main]
async fn main() -> BoxResult<()> {
    print_header();
    let config = cli::parse_args().await?;
    let log_level = match config.optional().debug {
        Some(true) => log::LevelFilter::Debug,
        _ => log::LevelFilter::Warn,
    };
    logger::init(log_level)?;
    info!("Config: {}", &config);
    if mlc::run(&config).await.is_err() {
        process::exit(1);
    } else {
        process::exit(0);
    }
}
