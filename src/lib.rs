/*
 * SPDX-FileCopyrightText: 2019 - 2024 Armin Becher <becherarmin@gmail.com>
 * SPDX-FileCopyrightText: 2022 - 2025 Robin Vobruba <hoijui.quaero@gmail.com>
 *
 * SPDX-License-Identifier: MIT
 */

pub mod config;
pub mod link_validator;

use crate::config::Config;
use crate::config::WinPathSepStrategy;
use crate::link_validator::resolve_target_link;
use cli_utils::path_buf::PathBuf;
use futures::{StreamExt, stream};
use git_version::git_version;
use link_validator::LinkCheckResult;
use log::info;
use mle::BoxResult;
use mle::ColoredString;
use mle::link::FileSystemLoc;
use mle::link::FileSystemTarget;
use mle::link::Link;
use mle::link::Locator;
use mle::link::Target;
use std::collections::HashMap;
use std::env;
use std::fmt::Write;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::sync::Arc;
use std::vec;
use tokio::sync::Mutex;
use tokio::time::{Duration, Instant, sleep_until};

pub const VERSION: &str = git_version!(cargo_prefix = "", fallback = "unknown");

#[derive(Debug, Clone)]
struct FinalResult {
    target: Target,
    result_code: LinkCheckResult,
}

// #[derive(Hash, PartialEq, Eq, Clone, Debug)]
// struct Target {
//     target: String,
//     link_type: LinkType,
// }

async fn find_all_links(config: &Config) -> Vec<Link> {
    // let mle_state = mle::state::State::new(config.extractor_cfg);
    let (links, anchors, errors) = mle::find_all_links(&config.extractor_cfg).await;
    // mle_state.
    // let mut files: Vec<MarkupFile> = Vec::new();
    // file_traversal::find(config, &mut files);
    // let mut links = vec![];
    // for file in files {
    //     links.append(&mut mle::extractors::find_links(&file, config)?.links);
    // }
    links
}

fn find_git_ignored_files() -> Option<Vec<PathBuf>> {
    let output = Command::new("git")
        .arg("ls-files")
        .arg("--ignored")
        .arg("--others")
        .arg("--exclude-standard")
        .output()
        .expect("Failed to execute 'git' command");

    if output.status.success() {
        let ignored_files = String::from_utf8(output.stdout)
            .expect("Invalid UTF-8 sequence")
            .lines()
            .filter(|line| {
                std::path::Path::new(line).extension().is_some_and(|ext| {
                    ext.eq_ignore_ascii_case("md") || ext.eq_ignore_ascii_case("html")
                })
            })
            .filter_map(|line| fs::canonicalize(Path::new(line.trim())).ok())
            .map(Into::into)
            .collect::<Vec<_>>();
        Some(ignored_files)
    } else {
        eprintln!(
            "git ls-files command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        None
    }
}

fn find_git_untracked_files() -> Option<Vec<PathBuf>> {
    let output = Command::new("git")
        .arg("ls-files")
        .arg("--others")
        .arg("--exclude-standard")
        .output()
        .expect("Failed to execute 'git' command");

    if output.status.success() {
        // TODO de-duplicate with last function
        let ignored_files = String::from_utf8(output.stdout)
            .expect("Invalid UTF-8 sequence")
            .lines()
            .filter(|line| {
                std::path::Path::new(line).extension().is_some_and(|ext| {
                    ext.eq_ignore_ascii_case("md") || ext.eq_ignore_ascii_case("html")
                })
            })
            .filter_map(|line| fs::canonicalize(Path::new(line.trim())).ok())
            .map(Into::into)
            .collect::<Vec<_>>();
        Some(ignored_files)
    } else {
        eprintln!(
            "git ls-files command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        None
    }
}

// fn print_link(link: &Link, status_code: &ColoredString, msg: &str, error_channel: bool) {
fn print_link(
    target: &Target,
    source: &Locator,
    status_code: &ColoredString,
    msg: &str,
    error_channel: bool,
) {
    let link_str = format!(
        "[{:^4}] {} ({}, {}) => {} - {}",
        status_code, source.file, source.pos.line, source.pos.column, target, msg
    );
    if error_channel {
        eprintln!("{link_str}");
    } else {
        println!("{link_str}");
    }
}

fn debug_print_link(target: &Target, source: &Locator, msg: &str) {
    println!(
        "::warning file={},line={},col={},title=link checker warning::{}. {}",
        source.file, source.pos.line, source.pos.column, target, msg
    );
}

fn print_result(result: &FinalResult, links: &HashMap<Target, Vec<Locator>>) {
    for locator in &links[&result.target] {
        let code = &result.result_code;
        //match &result.result_code {
        //    LinkCheckResult::Ok => {
        //        print_helper(link, &"OK".green(), "", false);
        //    }
        //    LinkCheckResult::NotImplemented(msg) | LinkCheckResult::Warning(msg) => {
        //        print_helper(link, &"Warn".yellow(), msg, false);
        //    }
        //    LinkCheckResult::Ignored(msg) => {
        //        print_helper(link, &"Skip".green(), msg, false);
        //    }
        //    LinkCheckResult::Failed(msg) => {
        //        print_helper(link, &"Err".red(), msg, true);
        //    }
        //}
        print_link(
            &result.target,
            locator,
            code.status_code(),
            code.msg(),
            code.has_issue(),
        );
    }
}

pub async fn run(config: &Config) -> BoxResult<()> {
    let links = find_all_links(config).await;
    // This groups all links that have the same target **file/location**,
    // disregarding the anker/fragment.
    let mut link_target_groups: HashMap<Target, Vec<Locator>> = HashMap::new();

    // let mut skipped = 0;

    // TODO include this? Maybe not, because it is already in mle (is it/should it be?)
    // let ignore_links = config
    //     .optional
    //     .ignore_links
    //     .as_ref()
    //     .map_or_else(Vec::new, |s| s.iter().map(|m| WildMatch::new(m)).collect());

    // TODO uncomment this, but move it to mle?
    //let git_ignored_files: Option<Vec<PathBuf>> = if config.optional.git_ignore.is_some() {
    //    let files = find_git_ignored_files();
    //    debug!("Found git_ignored files: {files:?}");
    //    files
    //} else {
    //    None
    //};
    //let is_git_ignore_enabled = git_ignored_files.is_some();

    // TODO uncomment this, but move it to mle?
    //let git_untracked_files: Option<Vec<PathBuf>> = if config.optional.git_untracked.is_some() {
    //    let files = find_git_untracked_files();
    //    debug!("Found git_untracked files: {files:?}");
    //    files
    //} else {
    //    None
    //};
    //let is_git_untracked_enabled = git_untracked_files.is_some();

    //let mut broken_references: Vec<BrokenExtractedLink> = vec![];
    for link in links {
        // match link {
        //     Ok(link) => {
        //let canonical_link_source = match fs::canonicalize(&link.source) {
        //    Ok(path) => path,
        //    Err(e) => {
        //        warn!(
        //            "Failed to canonicalize link source: {}. Error: {:?}",
        //            link.source, e
        //        );
        //        continue;
        //    }
        //};

        // TODO uncomment this, but move it to mle?
        //if is_git_ignore_enabled {
        //    if let Some(ref gif) = git_ignored_files {
        //        if gif.iter().any(|path| path == &canonical_link_source) {
        //            print_helper(
        //                link,
        //                &"Skip".green(),
        //                "Ignore link because it is ignored by git.",
        //                false,
        //            );
        //            skipped += 1;
        //            continue;
        //        }
        //    }
        //}

        // TODO uncomment this, but move it to mle?
        //if is_git_untracked_enabled {
        //    if let Some(ref gif) = git_untracked_files {
        //        if gif.iter().any(|path| path == &canonical_link_source) {
        //            print_helper(
        //                link,
        //                &"Skip".green(),
        //                "Ignore link because it is untracked by git.",
        //                false,
        //            );
        //            skipped += 1;
        //            continue;
        //        }
        //    }
        //}
        // }

        // TODO include this? Maybe not, because it is already in mle (is it/should it be?)
        // if ignore_links.iter().any(|m| m.matches(&link.target.)) {
        //     print_helper(
        //         link,
        //         &"Skip".green(),
        //         "Ignore link because of ignore-links option.",
        //         false,
        //     );
        //     skipped += 1;
        //     continue;
        // }

        if !config
            .optional
            .do_not_warn_for_absolute_paths
            .unwrap_or_default()
        {
            if let Target::FileUrl(_) = &link.target {
                log::warn!("Detected link with file URL: {link}");
            } else if let Target::FileSystem(FileSystemTarget {
                file: FileSystemLoc::Absolute(_),
                ..
            }) = &link.target
            {
                log::warn!("Detected link with absolute file-system path: {link}");
            }
        }

        log::error!("XXX test output 0");
        if let Some(action) = &config.optional.on_win_path_sep
            && !matches!(action, WinPathSepStrategy::Accept)
            && let Target::FileSystem(FileSystemTarget { file, .. }) = &link.target
            && file.get_raw().contains('\\')
        {
            match action {
                WinPathSepStrategy::Accept => (),
                WinPathSepStrategy::Warn => {
                    log::warn!(r"Detected link with Windows path separator(s) ('\'): {link}");
                }
                WinPathSepStrategy::Error => {
                    log::error!(r"Detected link with Windows path separator(s) ('\'): {link}");
                    // TODO FIXME Also return an error somehow
                    continue;
                }
            }
        }

        let target = resolve_target_link(&link, config)?;
        link_target_groups
            .entry(target)
            .or_default()
            .push(link.source);
        // match link_target_groups.get_mut(&t) {
        //     Some(v) => v.push(link.clone()),
        //     None => {
        //         link_target_groups.insert(t, vec![link.clone()]);
        //     }
        // }
        //Err(broken_reference) => {
        //    broken_references.push(broken_reference.clone());
        //}
        // }
    }

    let throttle = config.optional.throttle.unwrap_or_default() > 0;
    info!("Throttle HTTP requests to same host: {throttle:?}");
    let waits = Arc::new(Mutex::new(HashMap::new()));
    // See also http://patshaughnessy.net/2020/1/20/downloading-100000-files-using-async-rust
    let mut buffered_stream = stream::iter(link_target_groups.keys())
        .map(|target| {
            let waits = waits.clone();
            async move {
                if throttle && let Target::Http(target_url) = target {
                    // let parsed = match Url::parse(target_url) {
                    //     Ok(parsed) => parsed,
                    //     Err(error) => {
                    //         return FinalResult {
                    //             target: target.clone(),
                    //             result_code: LinkCheckResult::Failed(format!(
                    //                 "Could not parse URL type. Err: {error:?}"
                    //             )),
                    //         }
                    //     }
                    // };
                    let host = match target_url.host_str() {
                        Some(host) => host.to_string(),
                        None => {
                            return FinalResult {
                                target: target.clone(),
                                result_code: LinkCheckResult::Failed(
                                    "Failed to determine host".to_string(),
                                ),
                            };
                        }
                    };
                    let mut waits = waits.lock().await;

                    let mut wait_until: Option<Instant> = None;
                    let next_wait = match waits.get(&host) {
                        Some(old) => {
                            wait_until = Some(*old);
                            *old + Duration::from_millis(
                                config.optional.throttle.unwrap_or_default().into(),
                            )
                        }
                        None => {
                            Instant::now()
                                + Duration::from_millis(
                                    config.optional.throttle.unwrap_or_default().into(),
                                )
                        }
                    };
                    waits.insert(host, next_wait);
                    drop(waits);

                    if let Some(deadline) = wait_until {
                        sleep_until(deadline).await;
                    }
                }

                let result_code = link_validator::check(target, config).await;

                FinalResult {
                    target: target.clone(),
                    result_code,
                }
            }
        })
        .buffer_unordered(config::DEFAULT_PARALLEL_REQUESTS);

    let mut oks = 0;
    let mut skipped = 0;
    let mut warnings = 0;
    let mut errors = vec![];

    let is_github_runner_env = env::var("GITHUB_ENV").is_ok();
    if is_github_runner_env {
        info!("Running in github environment. Print errors and warnings as workflow commands");
    }

    let mut process_result = |result| {
        print_result(&result, &link_target_groups);
        match &result.result_code {
            LinkCheckResult::Ok => {
                oks += link_target_groups[&result.target].len();
            }
            LinkCheckResult::NotImplemented(msg) | LinkCheckResult::Warning(msg) => {
                warnings += link_target_groups[&result.target].len();
                if is_github_runner_env {
                    for source in &link_target_groups[&result.target] {
                        debug_print_link(&result.target, source, msg);
                    }
                }
            }
            LinkCheckResult::Ignored(_) => {
                skipped += link_target_groups[&result.target].len();
            }
            LinkCheckResult::Failed(msg) => {
                errors.push(result.clone());
                if is_github_runner_env {
                    for source in &link_target_groups[&result.target] {
                        debug_print_link(&result.target, source, msg);
                    }
                }
            }
        }
    };

    while let Some(result) = buffered_stream.next().await {
        process_result(result);
    }

    println!();
    let error_sum: usize = errors
        .iter()
        .map(|e| link_target_groups[&e.target].len())
        .sum();
    let sum = skipped + error_sum + warnings + oks;
    println!("Result ({sum} links):");
    println!();
    println!("OK       {oks}");
    println!("Skipped  {skipped}");
    println!("Warnings {warnings}");
    println!("Errors   {error_sum}");
    println!();

    // return Ok(());
    if errors.is_empty() {
        Ok(())
    } else {
        let mut error_msg = String::new();
        writeln!(error_msg).unwrap();
        writeln!(error_msg, "The following links could not be resolved:").unwrap();
        writeln!(error_msg).unwrap();
        for res in errors {
            for source in &link_target_groups[&res.target] {
                writeln!(error_msg, "{source}:{:#?}", res.target).unwrap();
            }
        }
        writeln!(error_msg).unwrap();
        Err(error_msg.into())
    }
}
