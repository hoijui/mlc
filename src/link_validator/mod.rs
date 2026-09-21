/*
 * SPDX-FileCopyrightText: 2019 - 2024 Armin Becher <becherarmin@gmail.com>
 * SPDX-FileCopyrightText: 2022 - 2025 Robin Vobruba <hoijui.quaero@gmail.com>
 *
 * SPDX-License-Identifier: MIT
 */

mod file_system;
mod http;
mod mail;

use std::sync::LazyLock;

use crate::Config;
use crate::link_validator::file_system::check_filesystem;
use crate::link_validator::http::check_http;
use cli_utils::BoxResult;
use log::info;
use mail::check_mail;
use mle::ColoredString;
use mle::Colorize;
use mle::link::Link;
use mle::link::Target;
use wildmatch::WildMatch;

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum LinkCheckResult {
    Ok,
    Failed(String),
    Warning(String),
    Ignored(String),
    NotImplemented(String),
}

impl LinkCheckResult {
    #[must_use]
    pub fn msg(&self) -> &'_ str {
        match self {
            Self::Ok => "",
            Self::Failed(msg)
            | Self::Warning(msg)
            | Self::Ignored(msg)
            | Self::NotImplemented(msg) => msg,
        }
    }

    /**
     * Returns a status code like "OK", "Warn" or "Err",
     * combined with shell escapes for coloring it.
     */
    #[must_use]
    pub fn status_code(&self) -> &'static ColoredString {
        static CODE_OK: LazyLock<ColoredString> = LazyLock::new(|| "OK".green());
        static CODE_WARN: LazyLock<ColoredString> = LazyLock::new(|| "Warn".yellow());
        static CODE_SKIP: LazyLock<ColoredString> = LazyLock::new(|| "Skip".green());
        static CODE_ERR: LazyLock<ColoredString> = LazyLock::new(|| "Err".red());
        match self {
            Self::Ok => &CODE_OK,
            Self::NotImplemented(_) | Self::Warning(_) => &CODE_WARN,
            Self::Ignored(_) => &CODE_SKIP,
            Self::Failed(_) => &CODE_ERR,
        }
    }

    #[must_use]
    pub const fn is_err(&self) -> bool {
        matches!(self, Self::Failed(_))
    }

    #[must_use]
    pub const fn has_issue(&self) -> bool {
        match self {
            Self::Ok | Self::Ignored(..) => false,
            Self::Failed(..) | Self::Warning(..) | Self::NotImplemented(..) => true,
        }
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
pub fn resolve_target_link(link: &Link, config: &Config) -> BoxResult<mle::link::Target> {
    if link.target.is_file_system() {
        file_system::resolve_target_link(link, config)
    } else {
        Ok(link.target.clone())
    }
}

const EMPTY_VEC: Vec<WildMatch> = vec![];

pub async fn check(link_target: &Target, config: &Config) -> LinkCheckResult {
    info!("Check link {link_target}.");
    match link_target {
        Target::Ftp(url) => LinkCheckResult::NotImplemented(format!(
            "Link type for '{url}' (FTP) is not supported yet and cannot be checked.",
        )),
        Target::UnknownUrlSchema(url) => LinkCheckResult::NotImplemented(format!(
            "Link type '{url}' (unknown) is not implemented yet and cannot be checked."
        )),
        Target::EMail(url) => check_mail(url),
        Target::Http(url) => {
            if config.optional.offline.unwrap_or_default() {
                LinkCheckResult::Ignored(
                    "Ignore web (HTTP) link, because of the offline flag.".to_string(),
                )
            } else {
                check_http(
                    url,
                    config
                        .optional()
                        .do_not_warn_for_redirect_to
                        .as_ref()
                        .unwrap_or(&EMPTY_VEC),
                )
                .await
            }
        }
        Target::FileSystem(fs_target) => check_filesystem(fs_target, config).await,
        Target::FileUrl(..) => todo!(), // TODO
        Target::Invalid(..) => todo!(), // TODO
    }
}
