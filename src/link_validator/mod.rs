mod file_system;
mod http;
mod mail;

use std::sync::LazyLock;

use crate::link_validator::file_system::check_filesystem;
use crate::link_validator::http::check_http;
use crate::Config;
use colored::ColoredString;
use colored::Colorize;
use log::info;
use mail::check_mail;
use mle::link::Link;
use mle::link::Target;

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

pub async fn resolve_target_link(link: &Link, config: &Config) -> Target {
    if link.target.is_file_system() {
        file_system::resolve_target_link(link, config).await
    } else {
        link.target.clone()
    }
}

pub async fn check(link_target: &Target, config: &Config) -> LinkCheckResult {
    info!("Checking link '{}' ...", &link_target);
    match link_target {
        Target::Ftp(..) | Target::UnknownUrlSchema(..) => LinkCheckResult::NotImplemented(format!(
            "Checking of link type {link_target:#?} is not implemented (yet).",
        )),
        Target::EMail(url) => check_mail(url),
        Target::Http(url) => {
            if config.optional.offline.unwrap_or_default() {
                LinkCheckResult::Ignored("Ignore web link because of the offline flag.".to_string())
            } else {
                check_http(url).await
            }
        }
        Target::FileSystem(fs_target) => check_filesystem(fs_target, config).await,
        Target::FileUrl(..) => todo!(), // TODO
        Target::Invalid(..) => todo!(), // TODO
    }
}
