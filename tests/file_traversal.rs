/*
 * SPDX-FileCopyrightText: 2019 - 2022 Armin Becher <becherarmin@gmail.com>
 * SPDX-FileCopyrightText: 2023 - 2025 Robin Vobruba <hoijui.quaero@gmail.com>
 *
 * SPDX-License-Identifier: MIT
 */

#[cfg(test)]
mod helper;

use async_std::fs;
use clap::ValueEnum;
use cli_utils::path_buf::PathBuf;
use mle::markup;
use mle::markup::Type as MarkupType;

use crate::helper::benches_dir;

#[tokio::test]
async fn find_markdown_files() {
    let markup_types = MarkupType::value_variants().to_vec();
    let root = benches_dir().join("benchmark/markdown/md_file_endings");
    let ignore_paths = vec![];
    let markup_files = markup::Type::find(root.as_path().into(), markup_types, ignore_paths)
        .await
        .unwrap();
    assert_eq!(markup_files.len(), 12);
}

#[tokio::test]
async fn empty_folder() {
    let markup_types = vec![MarkupType::Markdown];
    let root = PathBuf::from("./target/empty");
    if !root.exists().await {
        fs::create_dir(&root).await.unwrap();
    }
    let ignore_paths = vec![];
    let markup_files = markup::Type::find(root.as_path(), markup_types, ignore_paths)
        .await
        .unwrap();
    assert!(markup_files.is_empty());
}
