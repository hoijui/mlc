/*
 * SPDX-FileCopyrightText: 2019 - 2020 Armin Becher <becherarmin@gmail.com>
 * SPDX-FileCopyrightText: 2020 Alex Melville <Melvillian@users.noreply.github.com>
 * SPDX-FileCopyrightText: 2022 - 2025 Robin Vobruba <hoijui.quaero@gmail.com>
 *
 * SPDX-License-Identifier: MIT
 */

use std::sync::Arc;

use clap::ValueEnum;
use mlc::{Config, OptionalConfig};
#[cfg(test)]
use mle::extractors::find_links;
use mle::{
    link::{FileLoc, FileSystemLoc, Position},
    markup::{Content, File as MarkupFile, Type as MarkupType},
    path_buf::PathBuf,
};

#[tokio::test]
async fn no_links() {
    let directory: PathBuf = "./benches/benchmark/markdown/no_links/".into();
    let file_path = directory.join("no_links.md");
    let file = MarkupFile {
        markup_type: MarkupType::Markdown,
        locator: Arc::new(FileLoc::System(FileSystemLoc::Absolute(file_path.clone()))),
        content: Content::LocalFile(file_path.clone()),
        start: Position::new(),
    };
    let config = Config::new(
        directory.clone(),
        mle::Config {
            files_and_dirs: vec![file_path.clone()],
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
    let result = find_links(&file, &config.extractor_cfg()).await.unwrap();
    assert!(result.links.is_empty());
}

#[tokio::test]
async fn some_links() {
    let directory: PathBuf = "./benches/benchmark/markdown/many_links/".into();
    let file_path = directory.join("many_links.md");
    let file = MarkupFile {
        markup_type: MarkupType::Markdown,
        locator: Arc::new(FileLoc::System(FileSystemLoc::Absolute(file_path.clone()))),
        content: Content::LocalFile(file_path.clone()),
        start: Position::new(),
    };
    let config = Config::new(
        directory.clone(),
        mle::Config {
            files_and_dirs: vec![file_path.clone()],
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
    let result = find_links(&file, &config.extractor_cfg()).await.unwrap();
    assert_eq!(result.links.len(), 11);
}
