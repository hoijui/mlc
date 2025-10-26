/*
 * SPDX-FileCopyrightText: 2020 - 2024 Armin Becher <becherarmin@gmail.com>
 * SPDX-FileCopyrightText: 2022 - 2025 Robin Vobruba <hoijui.quaero@gmail.com>
 *
 * SPDX-License-Identifier: MIT
 */

use crate::Config;
use crate::link_validator::LinkCheckResult;
use async_std::fs::canonicalize;
use async_std::path::PathBuf as AsyncPathBuf;
use async_std::stream::StreamExt;
use async_walkdir::WalkDir;
use log::{debug, info, warn};
use mle::link::FileSystemTarget;
use mle::link::Link;
use mle::path_buf::PathBuf;
use std::borrow::Cow;
use std::path::MAIN_SEPARATOR;
use std::sync::LazyLock;

pub async fn check_filesystem(target: &FileSystemTarget, config: &Config) -> LinkCheckResult {
    let target: PathBuf = target
        .file
        .to_absolute_path(config.rel_path_base.as_ref())
        .into_owned();
    debug!("Absolute target path: '{target:?}'");
    if target.exists().await {
        LinkCheckResult::Ok
    } else if !config.optional.match_file_extension.unwrap_or_default()
        && target.extension().is_none()
    {
        // Check if file exists ignoring the file extension
        let Some(target_file_name) = target.file_name() else {
            return LinkCheckResult::Failed("Target path not found.".to_string());
        };
        let Some(target_parent) = target.parent() else {
            return LinkCheckResult::Failed("Target parent not found.".to_string());
        };
        debug!("Check if file ignoring the extension exists.");
        if target_parent.exists().await {
            debug!("Parent {target_parent:?} exists. Search dir for file ignoring the extension.");
            let mut walkdir = WalkDir::new(target_parent);
            while let Some(entry_res) = walkdir.next().await {
                if let Ok(entry) = entry_res
                    && entry.path().iter().count() == 1
                    && let Ok(file_type) = entry.file_type().await
                    && !file_type.is_dir()
                {
                    let mut file_on_system = entry.path();
                    file_on_system.set_extension("");
                    match file_on_system.file_name() {
                        Some(file_name) => {
                            if target_file_name == file_name {
                                info!("Found file {}", file_on_system.display());
                                return LinkCheckResult::Ok;
                            }
                        }
                        None => {
                            return LinkCheckResult::Failed(
                                "Target filename not found.".to_string(),
                            );
                        }
                    }
                }
            }
            LinkCheckResult::Failed("Target not found.".to_string())
        } else {
            LinkCheckResult::Failed("Target not found.".to_string())
        }
    } else {
        LinkCheckResult::Failed("Target filename not found.".to_string())
    }
}

pub fn resolve_target_link(link: &Link, config: &Config) -> mle::link::Target {
    if let Some(anchor) = link.target.fragment() {
        warn!(
            "Strip everything after #. The chapter (aka anchor aka fragment) part '{anchor}' is not checked.",
        );
    }
    match &link.target {
        mle::link::Target::Http(_)
        | mle::link::Target::Ftp(_)
        | mle::link::Target::EMail(_)
        | mle::link::Target::FileUrl(_)
        | mle::link::Target::UnknownUrlSchema(_)
        | mle::link::Target::Invalid(_) => panic!(
            "FS Link validator called with wrong target type {:#?}",
            link.target
        ),
        mle::link::Target::FileSystem(fs_target) => {
            let abs_path = match &fs_target.file {
                mle::link::FileSystemLoc::Relative(path) => {
                    let path_str = path.to_string();
                    let abs_path = if path_str.starts_with('/') || path_str.starts_with('\\') {
                        config.rel_path_base.join(&path_str[1..])
                    } else {
                        path.to_path(&config.rel_path_base).into()
                    };
                    Cow::Owned(abs_path)
                }
                mle::link::FileSystemLoc::Absolute(path) => {
                    let mut abs_path = Cow::Borrowed(path);
                    if let Some(root_dir) = &config.optional.root_dir.as_ref() {
                        println!("XXX 1 '{root_dir}'");
                        let path_str = path.as_os_str().display().to_string();
                        // Yes! We also want to redirect absolute paths to the provided root dir!
                        if path_str.starts_with('/') || path_str.starts_with('\\') {
                            println!("XXX 2");
                            // TODO HACK Squishy!
                            abs_path = Cow::Owned(root_dir.join(&path_str[1..]));
                        }
                    }
                    println!("XXX Resolved absolute path '{path}' to '{abs_path}'");
                    abs_path
                }
            };
            debug!("Checking file system link target '{:?}' ...", link.target);
            // let abs_path = absolute_target_path(abs_path, &link.target)
            //     .await
            //     .to_str()
            //     .expect("Could not resolve target path")
            //     .to_string();
            // Remove verbatim path identifier which causes trouble on windows when using ../../ in paths
            let abs_path = abs_path
                .strip_prefix(r"\\?\")
                .map(|path| PathBuf::from(path.into()))
                .unwrap_or_else(|_err| abs_path.into_owned())
                .display()
                .to_string();
            mle::link::Target::FileSystem(mle::link::FileSystemTarget {
                file: mle::link::FileSystemLoc::Absolute(PathBuf::from(abs_path.into())),
                anchor: fs_target.anchor.clone(),
            })
        }
    }
}

async fn absolute_target_path(source: &str, target: &AsyncPathBuf) -> AsyncPathBuf {
    static ROOT: LazyLock<AsyncPathBuf> =
        LazyLock::new(|| AsyncPathBuf::from(format!("{MAIN_SEPARATOR}").as_str()));
    if target.is_relative() {
        let abs_source = canonicalize(source)
            .await
            .unwrap_or_else(|_| panic!("Path '{source}' does not exist."));
        let new_target = target
            .strip_prefix(format!(".{MAIN_SEPARATOR}"))
            .map_or(target.as_path(), |t| t);
        let parent: AsyncPathBuf = abs_source
            .parent()
            .map_or_else(|| ROOT.clone(), AsyncPathBuf::from);
        parent.join(new_target)
    } else {
        target.clone()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use async_std::path::Path;

    #[tokio::test]
    async fn remove_dot() {
        let source = Path::new(file!())
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("benches")
            .join("benchmark");
        let target = Path::new("./script_and_comments.md").into();

        let path = absolute_target_path(source.to_str().unwrap(), &target).await;

        let path_str = path.display().to_string();
        println!("{path_str}");
        assert_eq!(path_str.matches('.').count(), 1);
    }
}
