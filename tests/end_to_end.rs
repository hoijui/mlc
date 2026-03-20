/*
 * SPDX-FileCopyrightText: 2019 - 2022 Armin Becher <becherarmin@gmail.com>
 * SPDX-FileCopyrightText: 2023 - 2025 Robin Vobruba <hoijui.quaero@gmail.com>
 *
 * SPDX-License-Identifier: MIT
 */

#[cfg(test)]
mod helper;

use cli_utils::StreamIdent;
use helper::benches_dir;
use mlc::Config;
use mlc::OptionalConfig;
use mle::markup;
use mle::markup::Type as MarkupType;
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
        "./benches/benchmark/markdown/ignore_me_dir"
            .try_into()
            .unwrap(),
    ];
    let ignore_links = vec![wildmatch::WildMatch::new("./doc/broken-local-link.doc")];
    let markup_files = markup::Type::find(root.as_path().into(), markup_types, ignore_paths)
        .await
        .unwrap();
    let config = Config::new(
        // directory.clone(),
        mle::Config {
            markup_files,
            // recursive: true,
            links: Some(StreamIdent::StdOut),
            anchors: Some(StreamIdent::StdOut),
            // ignore_paths: vec![],
            // ignore_links: vec![],
            ignore_links,
            // markup_types: MarkupType::value_variants().to_vec(),
            // result_format: mle::result::Type::Markdown,
            result_format: mle::result::Type::Json,
            result_extended: true,
            result_flush: true,
        },
        OptionalConfig {
            debug: None,
            do_not_warn_for_redirect_to: None,
            markup_types: Some(vec![MarkupType::Markdown]),
            offline: None,
            match_file_extension: None,
            throttle: None,
            // ignore_links: Some(ignore_links),
            ignore_links: None,
            // ignore_paths: Some(ignore_paths),
            ignore_paths: None,
            root_dir: None,
            git_ignore: None,
            git_untracked: None,
            csv_file: None,
        },
    )
    .await
    .unwrap();
    if let Err(err) = mlc::run(&config).await {
        panic!("Test with custom root failed. {err:?}");
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
    // let config = Config {
    //     markup_files,
    //     links: Some(StreamIdent::StdOut),
    //     anchors: Some(StreamIdent::StdOut),
    //     result_format: result::Type::Json,
    //     ..Default::default()
    // };

    let test_files = benches_dir().join("different_root");
    let csv_output = std::env::temp_dir().join("mlc_test_output.csv");
    let config = Config::new(
        // test_files.clone(),
        mle::Config {
            markup_files,
            links: Some(StreamIdent::StdOut),
            anchors: Some(StreamIdent::StdOut),
            result_format: mle::result::Type::Json,
            ..Default::default() // files_and_dirs: vec![test_files.clone().into()],
                                 // recursive: true,
                                 // links: Some(StreamIdent::StdOut),
                                 // anchors: Some(StreamIdent::StdOut),
                                 // ignore_paths: vec![],
                                 // ignore_links: vec![],
                                 // markup_types: MarkupType::value_variants().to_vec(),
                                 // result_format: mle::result::Type::Markdown,
                                 // result_extended: true,
                                 // result_flush: true,
        },
        OptionalConfig {
            debug: Some(true),
            do_not_warn_for_redirect_to: None,
            markup_types: Some(vec![MarkupType::Markdown]),
            offline: None,
            match_file_extension: None,
            ignore_links: None,
            ignore_paths: None,
            throttle: None,
            root_dir: Some(test_files.into()),
            git_ignore: None,
            git_untracked: None,
            csv_file: Some(csv_output.clone().into()),
        },
    )
    .await
    .unwrap();
    if let Err(err) = mlc::run(&config).await {
        panic!("Test with custom root failed. {err:?}");
    } else {
        // Check if the CSV file was created, but is empty except for the header
        let content = fs::read_to_string(csv_output).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0], "source,line,column,target");
    }
}

#[tokio::test]
async fn end_to_end_write_csv_file() {
    let test_files = benches_dir().join("benchmark/markdown/ignore_me.md");
    let csv_output = std::env::temp_dir().join("mlc_test_output.csv");
    let markup_files = vec![test_files.into()];
    let config = Config::new(
        // test_files.clone(),
        mle::Config {
            markup_files,
            links: Some(StreamIdent::StdOut),
            anchors: Some(StreamIdent::StdOut),
            result_format: mle::result::Type::Json,
            ..Default::default() // let config = Config::new(
                                 //     // test_files.clone(),
                                 //     mle::Config {
                                 //         files_and_dirs: vec![test_files.clone().into()],
                                 //         recursive: true,
                                 //         links: Some(StreamIdent::StdOut),
                                 //         anchors: Some(StreamIdent::StdOut),
                                 //         ignore_paths: vec![],
                                 //         ignore_links: vec![],
                                 //         markup_types: MarkupType::value_variants().to_vec(),
                                 //         result_format: mle::result::Type::Markdown,
                                 //         result_extended: true,
                                 //         result_flush: true,
        },
        OptionalConfig {
            debug: None,
            do_not_warn_for_redirect_to: None,
            markup_types: Some(vec![MarkupType::Markdown]),
            offline: None,
            match_file_extension: None,
            throttle: None,
            ignore_links: None,
            ignore_paths: None,
            root_dir: None,
            git_ignore: None,
            git_untracked: None,
            csv_file: Some(csv_output.clone().into()),
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
