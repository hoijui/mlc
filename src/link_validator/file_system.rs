/*
 * SPDX-FileCopyrightText: 2020 - 2024 Armin Becher <becherarmin@gmail.com>
 * SPDX-FileCopyrightText: 2022 - 2025 Robin Vobruba <hoijui.quaero@gmail.com>
 *
 * SPDX-License-Identifier: MIT
 */

use crate::Config;
use crate::link_validator::LinkCheckResult;
use async_std::stream::StreamExt;
use async_walkdir::WalkDir;
use bstr::ByteSlice;
use cli_utils::BoxResult;
use cli_utils::path_buf::PathBuf;
use log::{debug, info, warn};
use mle::link::FileSystemLoc;
use mle::link::FileSystemTarget;
use mle::link::Link;
use std::borrow::Cow;
use std::os::unix::ffi::OsStrExt;
use std::sync::Arc;

/// Checks if a link-target on the local file-system exists.
pub async fn check_filesystem(target: &FileSystemTarget, config: &Config) -> LinkCheckResult {
    // let target: PathBuf = target
    //     .file
    //     .to_absolute_path(config.rel_path_base.as_ref())
    //     .into_owned(); // NOTE The base for this has to be the source file, not th base path
    let FileSystemLoc::Absolute(target_path) = &target.file else {
        panic!(
            "Programmer Error: No relative path (as a link target) \
should end up at the checking stage!"
        );
    };

    // We convert to a `bstr` here,
    // because standard OsStr(ing) is neither investigatable nor manipulatable.
    let rich_target_path = bstr::BStr::new(target_path.as_os_str().as_bytes());
    let clean_target_path = if rich_target_path.contains(&b'\\') {
        Cow::Owned(rich_target_path.replace("\\", "/").into())
    } else {
        Cow::Borrowed(rich_target_path)
    };
    let Ok(target_path_os_str) = clean_target_path.to_os_str() else {
        return LinkCheckResult::Failed(
            "Failed to convert cleaned target path back to an OsStr.".to_string(),
        );
    };
    let target_path = PathBuf::from(std::path::PathBuf::from(target_path_os_str));

    debug!("Absolute target path: '{target_path:?}'");
    if target_path.exists().await {
        LinkCheckResult::Ok
    } else if !config.optional.match_file_extension.unwrap_or_default()
        && target_path.extension().is_none()
    {
        // Check if file exists ignoring the file extension
        let Some(target_file_name) = target_path.file_name() else {
            return LinkCheckResult::Failed("Target path not found.".to_string());
        };
        let Some(target_parent) = target_path.parent() else {
            return LinkCheckResult::Failed("Target parent not found.".to_string());
        };
        debug!("Check if file ignoring the extension exists.");
        if target_parent.exists().await {
            debug!("Parent {target_parent:?} exists. Search dir for file ignoring the extension.");
            let mut walk_dir = WalkDir::new(target_parent);
            while let Some(entry_res) = walk_dir.next().await {
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
        }
        LinkCheckResult::Failed("Target not found.".to_string())
    } else {
        LinkCheckResult::Failed("Target filename not found.".to_string())
    }
}

/// This performs a (kind of) canonicalization of the link target.
///
/// We call this in order to only check each link once,
/// even if it appears in different forms.
///
/// For example,
/// target `hello.html` in file `index.html`
/// and target `../hello.html` in file `sub-dir/sub-file.html`
/// will be canonicalized to the same target.
// pub fn resolve_target_link(link: &Link, config: &Config) -> mle::link::Target {
pub fn resolve_target_link(link: &Link, config: &Config) -> BoxResult<mle::link::Target> {
    if let Some(anchor) = link.target.fragment() {
        warn!(
            "We are stripping everything after '#'. \
The chapter (aka anchor aka fragment) part '{anchor}' is not checked.",
        );
    }

    // let base = Arc::clone(&link.source.file).canonical(&config.rel_path_base)?;
    // // let base = if let FileLoc::System(FileSystemLoc::Relative(rel_source_path)) =
    // //     &link.source.file.as_ref()
    // // {
    // //     Arc::new(FileLoc::System(FileSystemLoc::Absolute(
    // //         config.rel_path_base.join(rel_source_path.as_str()),
    // //     )))
    // // } else {
    // //     Arc::clone(&link.source.file)
    // // };
    // let base = base.parent()?;

    log::warn!("Relative path: '{}'", link.target);
    // log::warn!("Relative base path (for file x): '{base:?}'");
    let resolved = link
        .target
        .canonical(
            /*&base*/ config.optional.root_dir.is_some(),
            Arc::clone(&link.source.file),
            /*rel_source_path,*/ &config.rel_path_base,
        )
        .map(Cow::into_owned);
    log::warn!("Canonicalized path: '{resolved:#?}'");
    // eprintln!("Canonicalized path: '{:#?}'", resolved);
    resolved
    // match &link.target {
    //     mle::link::Target::Http(_)
    //     | mle::link::Target::Ftp(_)
    //     | mle::link::Target::EMail(_)
    //     | mle::link::Target::FileUrl(_)
    //     | mle::link::Target::UnknownUrlSchema(_)
    //     | mle::link::Target::Invalid(_) => {
    //         return Err(format!(
    //             "FS Link validator called with wrong target type {:#?}",
    //             link.target
    //         )
    //         .into());
    //     }
    //     mle::link::Target::FileSystem(fs_target) => {
    //         let abs_path = match &fs_target.file {
    //             mle::link::FileSystemLoc::Relative(relative_path) => {
    //                 let abs_path = link.source.file.join(relative_path.as_str())?; // FIXME This does not support OS-Str
    //                 // let path_str = path.to_string();
    //                 // let abs_path = if path_str.starts_with('/') || path_str.starts_with('\\') {
    //                 //     config.rel_path_base.join(&path_str[1..])
    //                 // } else {
    //                 //     path.to_path(&config.rel_path_base).into()
    //                 // };
    //                 // Cow::Owned(abs_path)
    //                 return Ok(mle::link::Target::FileSystem(mle::link::FileSystemTarget {
    //                     file: abs_path,
    //                     anchor: fs_target.anchor.clone(),
    //                 }));
    //             }
    //             mle::link::FileSystemLoc::Absolute(path) => {
    //                 let mut abs_path = Cow::Borrowed(path);
    //                 if let Some(root_dir) = &config.optional.root_dir.as_ref() {
    //                     println!("XXX 1 '{root_dir}'");
    //                     let path_str = path.as_os_str().display().to_string();
    //                     // Yes! We also want to redirect absolute paths to the provided root dir!
    //                     if path_str.starts_with('/') || path_str.starts_with('\\') {
    //                         println!("XXX 2");
    //                         // TODO HACK Squishy!
    //                         abs_path = Cow::Owned(root_dir.join(&path_str[1..]));
    //                     }
    //                 }
    //                 println!("XXX Resolved absolute path '{path}' to '{abs_path}'");
    //                 abs_path
    //             }
    //         };
    //         debug!("Checking file system link target '{:?}' ...", link.target);
    //         // let abs_path = absolute_target_path(abs_path, &link.target)
    //         //     .await
    //         //     .to_str()
    //         //     .expect("Could not resolve target path")
    //         //     .to_string();
    //         // Remove verbatim path identifier which causes trouble on windows when using ../../ in paths
    //         let abs_path = abs_path
    //             .strip_prefix(r"\\?\")
    //             .map(PathBuf::from)
    //             .unwrap_or_else(|_err| abs_path.into_owned())
    //             .display()
    //             .to_string();
    //         Ok(mle::link::Target::FileSystem(mle::link::FileSystemTarget {
    //             file: mle::link::FileSystemLoc::Absolute(
    //                 PathBuf::from_str(&abs_path).expect("Infallible"),
    //             ),
    //             anchor: fs_target.anchor.clone(),
    //         }))
    //     }
    // }
}

// async fn absolute_target_path(source: &str, target: &AsyncPathBuf) -> AsyncPathBuf {
//     static ROOT: LazyLock<AsyncPathBuf> =
//         LazyLock::new(|| AsyncPathBuf::from(format!("{MAIN_SEPARATOR}").as_str()));
//     if target.is_relative() {
//         let abs_source = canonicalize(source)
//             .await
//             .unwrap_or_else(|_| panic!("Path '{source}' does not exist."));
//         let new_target = target
//             .strip_prefix(format!(".{MAIN_SEPARATOR}"))
//             .map_or(target.as_path(), |t| t);
//         let parent: AsyncPathBuf = abs_source
//             .parent()
//             .map_or_else(|| ROOT.clone(), AsyncPathBuf::from);
//         parent.join(new_target)
//     } else {
//         target.clone()
//     }
// }

#[cfg(test)]
mod test {
    use crate::config::OptionalConfig;

    use super::*;
    use async_std::path::Path;
    use mle::link::{FileLoc, Position, Target};

    fn benchmark_dir() -> PathBuf {
        Path::new(file!())
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("benches")
            .join("benchmark")
            .into()
    }

    fn benchmark_dir_abs() -> PathBuf {
        std::env::current_dir()
            .unwrap()
            .join(benchmark_dir())
            .into()
    }

    fn new_local_link<P: AsRef<Path>>(source: P, target: P) -> Link {
        Link::new(
            Arc::new(FileLoc::from(source.as_ref())),
            Position::new(),
            target.as_ref().to_str().unwrap(),
        )
    }

    #[tokio::test]
    async fn test_resolve_target_link() {
        let sources_root = benchmark_dir_abs();
        let source = PathBuf::from("markdown/anchor_links_2.md");
        let target = PathBuf::from("anchor_links.md");
        let link = new_local_link(source, target);
        let config = Config::new(
            mle::Config {
                ..Default::default()
            },
            OptionalConfig {
                // root_dir: None,
                root_dir: Some(sources_root.clone()),
                ..Default::default()
            },
        )
        .await
        .unwrap();
        let resolved_target = resolve_target_link(&link, &config).unwrap();

        // let path_str = resolved_target.to_string();
        // println!("{path_str}");
        // assert_eq!(path_str, "markdown/anchor_links.md");

        let actual_target_file = match resolved_target {
            Target::FileSystem(fs_target) => match fs_target.file {
                FileSystemLoc::Absolute(path) => path,
                FileSystemLoc::Relative(_) => panic!("Expected absolute path"),
            },
            _ => panic!("Expected file system target"),
        };
        let expected_target_file = sources_root.join("markdown/anchor_links.md");

        assert_eq!(actual_target_file, expected_target_file);
    }

    // #[tokio::test]
    // async fn test_absolute_target_path() {
    //     let source = benchmark_dir();
    //     let target = PathBuf::from("./script_and_comments.md");

    //     let path = absolute_target_path(source.to_str().unwrap(), &target).await;

    //     let path_str = path.display().to_string();
    //     println!("{path_str}");
    //     assert_eq!(path_str.matches('.').count(), 1);
    // }
}
