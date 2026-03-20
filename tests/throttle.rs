/*
 * SPDX-FileCopyrightText: 2021 - 2022 Armin Becher <becherarmin@gmail.com>
 * SPDX-FileCopyrightText: 2023 - 2025 Robin Vobruba <hoijui.quaero@gmail.com>
 *
 * SPDX-License-Identifier: MIT
 */

#[cfg(test)]
mod helper;

use cli_utils::StreamIdent;
use helper::benches_dir;
use mlc::{Config, OptionalConfig};
use mle::markup::Type as MarkupType;
use std::time::{Duration, Instant};

const TEST_THROTTLE_MS: u32 = 100;
const TEST_URLS: u32 = 10;
const THROTTLED_TIME_MS: u64 = (TEST_THROTTLE_MS as u64) * ((TEST_URLS as u64) - 1);

// #[tokio::test] // TODO For this test, we need manually settable timeout first, to make sure that the timeout * 10 (== #URLs) is quite some less then the throttle time * 10, or even less then throttle time * 2!
async fn throttle_different_hosts() {
    let test_file = benches_dir().join("throttle").join("different_host.md");
    let config = Config::new(
        mle::Config {
            markup_files: vec![test_file.clone().into()],
            links: Some(StreamIdent::StdOut),
            anchors: Some(StreamIdent::StdOut),
            ignore_links: vec![],
            result_format: mle::result::Type::Markdown,
            result_extended: true,
            result_flush: true,
        },
        OptionalConfig {
            throttle: Some(TEST_THROTTLE_MS),
            markup_types: Some(vec![MarkupType::Markdown]),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    let start = Instant::now();
    mlc::run(&config).await.unwrap_or(());
    let duration = start.elapsed();
    assert!(duration < Duration::from_millis(THROTTLED_TIME_MS));
}

#[tokio::test]
async fn throttle_same_hosts() {
    let test_file = benches_dir().join("throttle").join("same_host.md");
    let config = Config::new(
        mle::Config {
            markup_files: vec![test_file.clone().into()],
            links: Some(StreamIdent::StdOut),
            anchors: Some(StreamIdent::StdOut),
            ignore_links: vec![],
            result_format: mle::result::Type::Markdown,
            result_extended: true,
            result_flush: true,
        },
        OptionalConfig {
            throttle: Some(TEST_THROTTLE_MS),
            markup_types: Some(vec![MarkupType::Markdown]),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    let start = Instant::now();
    mlc::run(&config).await.unwrap_or(());
    let duration = start.elapsed();
    assert!(duration > Duration::from_millis(THROTTLED_TIME_MS));
}

#[tokio::test]
async fn throttle_same_ip() {
    let test_file = benches_dir().join("throttle").join("same_ip.md");
    let config = Config::new(
        mle::Config {
            markup_files: vec![test_file.clone().into()],
            links: Some(StreamIdent::StdOut),
            anchors: Some(StreamIdent::StdOut),
            ignore_links: vec![],
            result_format: mle::result::Type::Markdown,
            result_extended: true,
            result_flush: true,
        },
        OptionalConfig {
            throttle: Some(TEST_THROTTLE_MS),
            markup_types: Some(vec![MarkupType::Markdown]),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    let start = Instant::now();
    mlc::run(&config).await.unwrap_or(());
    let duration = start.elapsed();
    assert!(duration > Duration::from_millis(THROTTLED_TIME_MS));
}
