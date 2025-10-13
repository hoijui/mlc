/*
 * SPDX-FileCopyrightText: 2019 - 2022 Armin Becher <becherarmin@gmail.com>
 * SPDX-FileCopyrightText: 2023 - 2025 Robin Vobruba <hoijui.quaero@gmail.com>
 *
 * SPDX-License-Identifier: MIT
 */

use async_std::fs;
use async_std::path::PathBuf as AsyncPathBuf;
use clap::ValueEnum;
use mlc::Config;
use mlc::OptionalConfig;
#[cfg(test)]
use mle::file_traversal;
use mle::markup::{File as MarkupFile, Type as MarkupType};
use std::path::Path;

#[tokio::test]
async fn find_markdown_files() -> Result<(), file_traversal::Error> {
    let path = Path::new("./benches/benchmark/markdown/md_file_endings").to_path_buf();
    let config = Config::new(
        path.clone().into(),
        mle::Config {
            files_and_dirs: vec![path.clone().into()],
            recursive: true,
            links: Some(None),
            anchors: Some(None),
            ignore_paths: vec![],
            ignore_links: vec![],
            markup_types: MarkupType::value_variants().to_vec(),
            result_format: mle::result::Type::Markdown,
            result_extended: true,
            result_flush: true,
        },
        OptionalConfig {
            markup_types: Some(vec![MarkupType::Markdown]),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    let mut result: Vec<MarkupFile> = Vec::new();

    file_traversal::find(&config.extractor_cfg(), &mut result).await?;
    assert_eq!(result.len(), 12);
    Ok(())
}

#[tokio::test]
async fn empty_folder() -> Result<(), file_traversal::Error> {
    let path = AsyncPathBuf::from("./target/empty");
    if !path.exists().await {
        fs::create_dir(&path).await.unwrap();
    }
    let config = Config::new(
        path.clone().into(),
        mle::Config {
            files_and_dirs: vec![path.clone().into()],
            recursive: true,
            links: Some(None),
            anchors: Some(None),
            ignore_paths: vec![],
            ignore_links: vec![],
            markup_types: MarkupType::value_variants().to_vec(),
            result_format: mle::result::Type::Markdown,
            result_extended: true,
            result_flush: true,
        },
        OptionalConfig {
            markup_types: Some(vec![MarkupType::Markdown]),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    let mut result: Vec<MarkupFile> = Vec::new();

    file_traversal::find(&config.extractor_cfg(), &mut result).await?;
    assert!(result.is_empty());
    Ok(())
}
