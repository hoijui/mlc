/*
 * SPDX-FileCopyrightText: 2019 - 2020 Armin Becher <becherarmin@gmail.com>
 * SPDX-FileCopyrightText: 2020 Alex Melville <Melvillian@users.noreply.github.com>
 * SPDX-FileCopyrightText: 2022 - 2026 Robin Vobruba <hoijui.quaero@gmail.com>
 *
 * SPDX-License-Identifier: MIT
 */

#[cfg(test)]
mod helper;

use cli_utils::path_buf::PathBuf;
use helper::benches_dir;
use mlc::config::{Config, OptionalConfig};
#[cfg(test)]
use mle::extractors::gather_links;
use mle::{
    link::{FileLoc, FileSystemLoc, Link, Position},
    markup::{Content, File as MarkupFile, Type as MarkupType},
};
use std::sync::Arc;

#[tokio::test]
async fn no_links() {
    let directory: PathBuf = benches_dir().join("benchmark/markdown/no_links/").into();
    let file_path = directory.join("no_links.md");
    let file = MarkupFile {
        markup_type: MarkupType::Markdown,
        locator: Arc::new(FileLoc::System(FileSystemLoc::Absolute(file_path.clone()))),
        content: Content::LocalFile(file_path.clone()),
        start: Position::new(),
    };
    let config = Config::new(
        mle::Config {
            markup_files: vec![file_path.clone()],
            links: true,
            anchors: true,
            ignore_links: vec![],
        },
        OptionalConfig {
            debug: Some(true),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    let result = gather_links(&file, config.extractor_cfg()).await.unwrap();
    assert_eq!(
        result.links,
        [] as [Link; 0],
        "no links should be found in file that has none"
    );
}

#[tokio::test]
async fn some_links() {
    let directory: PathBuf = benches_dir().join("benchmark/markdown/many_links/").into();
    let file_path = directory.join("many_links.md");
    let file = MarkupFile {
        markup_type: MarkupType::Markdown,
        locator: Arc::new(FileLoc::System(FileSystemLoc::Absolute(file_path.clone()))),
        content: Content::LocalFile(file_path.clone()),
        start: Position::new(),
    };
    let config = Config::new(
        mle::Config {
            markup_files: vec![file_path.clone()],
            links: true,
            anchors: true,
            ignore_links: vec![],
        },
        OptionalConfig {
            debug: Some(true),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    let result = gather_links(&file, config.extractor_cfg()).await.unwrap();
    assert_eq!(result.links.len(), 11);
}
