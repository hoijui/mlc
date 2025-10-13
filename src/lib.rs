/*
 * SPDX-FileCopyrightText: 2019 - 2022 Armin Becher <becherarmin@gmail.com>
 * SPDX-FileCopyrightText: 2022 - 2025 Robin Vobruba <hoijui.quaero@gmail.com>
 *
 * SPDX-License-Identifier: MIT
 */

#![warn(rust_2021_compatibility)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::cargo)]
#![warn(clippy::wildcard_enum_match_arm)]
//#![warn(clippy::string_slice)]
//#![warn(clippy::indexing_slicing)]
#![warn(clippy::clone_on_ref_ptr)]
#![warn(clippy::try_err)]
// #![warn(clippy::shadow_reuse)]
//#![warn(clippy::empty_structs_with_brackets)]
#![allow(clippy::else_if_without_else)]
#![allow(clippy::option_if_let_else)]
#![allow(clippy::clone_on_ref_ptr)]
#![warn(clippy::use_debug)]
//#![warn(clippy::print_stdout)]
//#![warn(clippy::print_stderr)]
#![allow(clippy::default_trait_access)]
// NOTE allowed because:
//      If the same regex is going to be applied to multiple inputs,
//      the pre-computations done by Regex construction
//      can give significantly better performance
//      than any of the `str`-based methods.
#![allow(clippy::trivial_regex)]
#![allow(clippy::struct_excessive_bools)]
#![allow(clippy::fn_params_excessive_bools)]
#![allow(clippy::cast_precision_loss)]

use crate::link_validator::resolve_target_link;
use async_std::fs::canonicalize;
pub use colored::*;
use futures::{StreamExt, stream};
use git_version::git_version;
use link_validator::LinkCheckResult;
use log::info;
use mle::ignore_path::IgnorePath;
use mle::link::Link;
use mle::link::Locator;
use mle::link::Target;
use mle::markup;
use mle::path_buf::PathBuf;
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::fmt;
use std::fmt::Write;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{Duration, Instant, sleep_until};
pub use wildmatch::WildMatch;

pub mod link_validator;

pub const VERSION: &str = git_version!(cargo_prefix = "", fallback = "unknown");

const PARALLEL_REQUESTS: usize = 20;

#[derive(Default, Debug, Deserialize)]
pub struct OptionalConfig {
    pub debug: Option<bool>,
    #[serde(rename(deserialize = "markup-types"))]
    pub markup_types: Option<Vec<markup::Type>>,
    pub offline: Option<bool>,
    #[serde(rename(deserialize = "match-file-extension"))]
    pub match_file_extension: Option<bool>,
    #[serde(rename(deserialize = "ignore-links"))]
    pub ignore_links: Option<Vec<String>>,
    #[serde(rename(deserialize = "ignore-path"))]
    pub ignore_paths: Option<Vec<IgnorePath>>,
    #[serde(rename(deserialize = "root-dir"))]
    pub root_dir: Option<PathBuf>,
    pub throttle: Option<u32>,
}

impl OptionalConfig {
    async fn canonicalize_root_dir(&mut self) -> Result<(), String> {
        if let Some(root_dir) = self.root_dir.as_ref() {
            match canonicalize(root_dir.as_path()).await {
                Ok(new_root) => {
                    self.root_dir = Some(new_root.into());
                }
                Err(err) => {
                    return Err(format!(
                        "Root path could not be converted to an absolute path. Does the directory exit? - '{err}'"
                    ));
                }
            }
        }
        Ok(())
    }
    // pub fn root_dir(&self) -> std::io::Result<async_std::path::PathBuf> {
    //     Ok(if let Some(root_dir) = self.root_dir {
    //         Cow::Borrowed(async_std::path::PathBuf::into(root_dir))
    //     } else {
    //         Cow::Owned(env::current_dir()?.into())
    //     })
    // }
    pub async fn eval_rel_path_base(&mut self) -> std::io::Result<PathBuf> {
        self.canonicalize_root_dir()
            .await
            .map_err(std::io::Error::other)?;
        Ok(if let Some(root_dir) = self.root_dir.clone() {
            root_dir
        } else {
            env::current_dir()?.into()
        })
    }
}

#[derive(Default, Debug, Deserialize)]
pub struct Config {
    pub(crate) directory: PathBuf,
    pub(crate) extractor_cfg: mle::Config,
    pub(crate) optional: OptionalConfig,
    #[serde(skip)]
    pub(crate) rel_path_base: PathBuf,
}

impl Config {
    pub async fn new(
        directory: PathBuf,
        extractor_cfg: mle::Config,
        mut optional: OptionalConfig,
    ) -> Result<Self, String> {
        optional.canonicalize_root_dir().await?;
        let rel_path_base = optional
            .eval_rel_path_base()
            .await
            .map_err(|err| err.to_string())?;
        Ok(Self {
            directory,
            extractor_cfg,
            optional,
            rel_path_base,
        })
    }

    pub fn directory(&self) -> &PathBuf {
        &self.directory
    }

    pub fn extractor_cfg(&self) -> &mle::Config {
        &self.extractor_cfg
    }

    pub fn optional(&self) -> &OptionalConfig {
        &self.optional
    }
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let ignore_str: Vec<String> = match &self.optional.ignore_links {
            Some(s) => s.iter().map(ToString::to_string).collect(),
            None => vec![],
        };
        let root_dir_str = match &self.optional.root_dir {
            Some(path) => path.as_os_str().to_str().unwrap_or(""),
            None => "",
        };
        let ignore_path_str: Vec<String> = match &self.optional.ignore_paths {
            Some(p) => p.iter().map(ToString::to_string).collect(),
            None => vec![],
        };
        let markup_types_str: Vec<String> = match &self.optional.markup_types {
            Some(p) => p.iter().map(|m| format!("{m:?}")).collect(),
            None => vec![],
        };
        write!(
            f,
            "
Debug: {:?}
Dir: {} 
Types: {:?} 
Offline: {}
MatchExt: {}
RootDir: {}
IgnoreLinks: {} 
IgnorePaths: {:?}
Throttle: {} ms",
            self.optional.debug.unwrap_or(false),
            self.directory.as_os_str().to_str().unwrap_or_default(),
            markup_types_str,
            self.optional.offline.unwrap_or_default(),
            self.optional.match_file_extension.unwrap_or_default(),
            root_dir_str,
            ignore_str.join(","),
            ignore_path_str,
            self.optional.throttle.unwrap_or(0)
        )
    }
}

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

// fn print_link(link: &Link, status_code: &colored::ColoredString, msg: &str, error_channel: bool) {
fn print_link(
    target: &Target,
    source: &Locator,
    status_code: &colored::ColoredString,
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
        print_link(
            &result.target,
            locator,
            code.status_code(),
            code.msg(),
            code.has_issue(),
        );
    }
}

pub async fn run(config: &Config) -> Result<(), String> {
    let links = find_all_links(config).await;
    // This groups all links that have the same target **file/location**,
    // disregarding the anker/fragment.
    let mut link_target_groups: HashMap<Target, Vec<Locator>> = HashMap::new();

    // let mut skipped = 0;

    // let ignore_links = config
    //     .optional
    //     .ignore_links
    //     .as_ref()
    //     .map_or_else(Vec::new, |s| s.iter().map(|m| WildMatch::new(m)).collect());
    for link in links {
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
        let target = resolve_target_link(&link, config).await;
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
        .buffer_unordered(PARALLEL_REQUESTS);

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

    if errors.is_empty() {
        Ok(())
    } else {
        let mut error_msg = String::new();
        writeln!(error_msg).unwrap();
        writeln!(error_msg, "The following links could not be resolved:").unwrap();
        writeln!(error_msg).unwrap();
        for res in errors {
            for source in &link_target_groups[&res.target] {
                writeln!(error_msg, "{source}").unwrap();
            }
        }
        writeln!(error_msg).unwrap();
        Err(error_msg)
    }
}
