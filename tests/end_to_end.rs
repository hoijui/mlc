/*
 * SPDX-FileCopyrightText: 2019 - 2022 Armin Becher <becherarmin@gmail.com>
 * SPDX-FileCopyrightText: 2023 - 2026 Robin Vobruba <hoijui.quaero@gmail.com>
 *
 * SPDX-License-Identifier: MIT
 */

#[cfg(test)]
mod helper;

use helper::benches_dir;
use mlc::config::Config;
use mlc::config::OptionalConfig;
use mle::markup;
use std::convert::TryInto;
use std::fs;
use std::path::MAIN_SEPARATOR;

#[tokio::test]
async fn end_to_end() {
    let markup_types = vec![markup::Type::Markdown];
    let root = benches_dir().join("benchmark");
    let ignore_paths = vec![
        "benches/benchmark/markdown/ignore_me.md"
            .try_into()
            .unwrap(),
        "benches/benchmark/markdown/link_ignore_file_extension.md"
            .try_into()
            .unwrap(),
        "./benches/benchmark/markdown/ignore_me_dir"
            .try_into()
            .unwrap(),
    ];
    let ignore_links = vec![wildmatch::WildMatch::new("./doc/broken-local-link.doc")];
    let markup_files = markup::Type::find(root.as_path().into(), markup_types, ignore_paths)
        .await
        .unwrap();
    let config = Config::new(
        mle::Config {
            markup_files,
            links: true,
            anchors: true,
            ignore_links,
        },
        OptionalConfig {
            debug: Some(true),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    if let Err(err) = mlc::run(&config).await {
        panic!("Test with custom root failed. {err}");
    }
}

#[tokio::test]
async fn end_to_end_different_root() {
    let markup_types = vec![markup::Type::Markdown];
    let root = benches_dir().join("different_root");
    let ignore_paths = vec![];
    let markup_files = markup::Type::find(root.as_path().into(), markup_types, ignore_paths)
        .await
        .unwrap();
    let csv_output = std::env::temp_dir().join("mlc_test_output.csv");
    let config = Config::new(
        mle::Config {
            markup_files,
            links: true,
            anchors: true,
            ..Default::default()
        },
        OptionalConfig {
            debug: Some(true),
            root_dir: Some(root.into()),
            csv_file: Some(csv_output.clone().into()),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    // let res = mlc::run(&config).await.unwrap();
    // if let Err(err) = mlc::run(&config).await {
    //     panic!("Test with custom root failed. {err}");
    // } else {
    //     // Check if the CSV file was created, but is empty except for the header
    //     let content = fs::read_to_string(csv_output).unwrap();
    //     let lines: Vec<&str> = content.lines().collect();
    //     assert_eq!(lines.len(), 1);
    //     assert_eq!(lines[0], "source,line,column,target");
    // }

    if let Err(err) = mlc::run(&config).await {
        panic!("Test with custom root failed. {err}");
    }
}

// #[tokio::test]
async fn end_to_end_write_csv_file() {
    let test_files = benches_dir().join("benchmark/markdown/ignore_me.md");
    let csv_output = std::env::temp_dir().join("mlc_test_output.csv");
    let markup_files = vec![test_files.into()];
    let config = Config::new(
        // test_files.clone(),
        mle::Config {
            markup_files,
            links: true,
            anchors: true,
            ..Default::default()
        },
        OptionalConfig {
            debug: Some(true),
            csv_file: Some(csv_output.clone().into()),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    if (mlc::run(&config).await).is_err() {
        let content = fs::read_to_string(csv_output).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 4);
        assert_eq!(lines[0], "source,line,column,target");
        for (i, line) in lines.iter().enumerate().skip(1) {
            assert_eq!(
                line,
                &format!(
                    "benches{MAIN_SEPARATOR}benchmark/markdown/ignore_me.md,{i},1,broken_Link",
                )
            );
        }
    } else {
        panic!("Should have detected errors");
    }
}
