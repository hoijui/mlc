/*
 * SPDX-FileCopyrightText: 2019 - 2024 Armin Becher <becherarmin@gmail.com>
 * SPDX-FileCopyrightText: 2022 - 2025 Robin Vobruba <hoijui.quaero@gmail.com>
 *
 * SPDX-License-Identifier: MIT
 */

use async_std::fs::canonicalize;
use cli_utils::path_buf::PathBuf;
use mle::BoxResult;
use serde::Deserialize;
use std::env;
use std::fmt;
use wildmatch::WildMatch;

pub const DEFAULT_PARALLEL_REQUESTS: usize = 20;

#[derive(Default, Debug, Deserialize)]
pub enum WinPathSepStrategy {
    #[default]
    Error,
    Warn,
    /// This means, it (`\`) will be fully accepted,
    /// and treated as if it were a UNIX/URL style file separator ('/').
    Accept,
}

#[derive(Default, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct OptionalConfig {
    /// If enabled, the log level is set to _debug_.
    pub debug: Option<bool>,
    /// When a link(-target) is an HTTP(S) URL,
    /// and the GET request returns a redirect code -
    /// suggesting that the page moved -
    /// we issue a warning by default.
    /// This setting disables that warning.
    pub do_not_warn_for_redirect_to: Option<Vec<WildMatch>>,
    /// When a link(-target) is an absolute file path,
    /// we issue a warning by default.
    /// This setting disables that warning.
    pub do_not_warn_for_absolute_paths: Option<bool>,
    /// What to do when a windows path separator ('\') is encountered
    /// in a link(-target) (that is a local file path).
    ///
    /// The default is, to treat it as an error,
    /// because such links are not portable.
    pub on_win_path_sep: Option<WinPathSepStrategy>,
    // /// TODO Deprecate(ed), because it happens before mlc (outside, with CLI tools)
    // pub markup_types: Option<Vec<markup::Type>>,
    /// If enabled, we skip checking web-links.
    /// Local file system links are still checked.
    pub offline: Option<bool>,
    /// If a link(-target) is an file path without an extension,
    /// and that file does not exist,
    /// then we search for files with the same name plus an extension,
    /// if this is enabled.
    ///
    /// NOTE:
    /// It is not recommended to use this,
    /// as such links will fail in most other scenarios.
    pub match_file_extension: Option<bool>,
    // /// TODO Deprecate(ed), because it is in mle now
    // pub ignore_links: Option<Vec<String>>,
    // /// TODO Deprecate(ed), because it happens before mlc (outside, with CLI tools)
    // pub ignore_paths: Option<Vec<IgnorePath>>,
    /// If set, each link(-target) that is an absolute file path,
    /// is prefixed with this absolute path when checking.
    ///
    /// NOTE:
    /// It is strongly recommended never to use
    /// absolute file path link(-target)s
    /// in production documents.
    pub root_dir: Option<PathBuf>,
    /// CSV format output file.
    ///
    /// DEPRECATED Not currently used.
    #[serde(rename(deserialize = "csv"))]
    pub csv_file: Option<PathBuf>,
    // /// Honor git-ignore, also ignoring those files when checking for links.
    // ///
    // /// DEPRECATED Not currently used.
    // pub git_ignore: Option<bool>,
    // /// Do not check documents that are not git tracked.
    // ///
    // /// DEPRECATED Not currently used.
    // pub git_untracked: Option<bool>,
    /// The amount of ms to throttle requests to the same host.
    /// `0` means no throttling (default: `0`).
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
    // pub(crate) directory: PathBuf,
    pub(crate) extractor_cfg: mle::config::Extractor,
    pub(crate) optional: OptionalConfig,
    /// The (absolute) base directory for link(-target)s
    /// that are relative file-system paths.
    ///
    /// If [`OptionalConfig::root_dir`] in [`Self::optional`] is set,
    /// it's value will be used here;
    /// else it is commonly going to be set to CWD/PWD.
    ///
    /// Note that actual resolving a link(-target)
    /// that is a relative file-system path
    /// also involves appending the source files parent to this value,
    /// before joining it with the link(-target).
    #[serde(skip)]
    pub(crate) rel_path_base: PathBuf,
}

impl Config {
    pub async fn new(
        // directory: impl Into<PathBuf>,
        extractor_cfg: mle::config::Extractor,
        mut optional: OptionalConfig,
    ) -> BoxResult<Self> {
        // optional.canonicalize_root_dir().await?;
        let rel_path_base = optional
            .eval_rel_path_base()
            .await
            .map_err(|err| err.to_string())?;
        Ok(Self {
            // directory: directory.into(),
            extractor_cfg,
            optional,
            rel_path_base,
        })
    }

    // #[must_use]
    // pub const fn directory(&self) -> &PathBuf {
    //     &self.directory
    // }

    #[must_use]
    pub const fn extractor_cfg(&self) -> &mle::config::Extractor {
        &self.extractor_cfg
    }

    #[must_use]
    pub const fn optional(&self) -> &OptionalConfig {
        &self.optional
    }
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // let ignore_str: Vec<String> = match &self.optional.ignore_links {
        //     Some(s) => s.iter().map(ToString::to_string).collect(),
        //     None => vec![],
        // };
        let root_dir_str = self
            .optional
            .root_dir
            .as_ref()
            .map_or("", |path| path.as_os_str().to_str().unwrap_or(""));
        // let ignore_path_str: Vec<String> = match &self.optional.ignore_paths {
        //     Some(paths) => paths.iter().map(ToString::to_string).collect(),
        //     None => vec![],
        // };
        let csv_file_str: Option<String> = self
            .optional
            .csv_file
            .as_ref()
            .map(|path| path.display().to_string());
        // let markup_types_str: Vec<String> = match &self.optional.markup_types {
        //     Some(types) => types.iter().map(|m| format!("{m:?}")).collect(),
        //     None => vec![],
        // };
        write!(
            f,
            "
Debug: {:?}
DoNotWarnForRedirectTo: {:?}
DoNotWarnForAbsolutePaths: {}
OnWindowsPathSeparator: {:?}
Offline: {}
MatchExt: {}
RootDir: {}
Throttle: {} ms
CSVFile: {:?}",
            // Types: {:?}
            // IgnoreLinks: {}
            // IgnorePaths: {:?}
            // git_ignore: {}
            // git_untracked: {}
            self.optional.debug.unwrap_or(false),
            // self.directory.as_os_str().to_str().unwrap_or_default(),
            self.optional.do_not_warn_for_redirect_to,
            self.optional
                .do_not_warn_for_absolute_paths
                .unwrap_or_default(),
            self.optional.on_win_path_sep,
            // markup_types_str,
            self.optional.offline.unwrap_or_default(),
            self.optional.match_file_extension.unwrap_or_default(),
            root_dir_str,
            // self.optional.git_ignore.unwrap_or_default(),
            // self.optional.git_untracked.unwrap_or_default(),
            // ignore_str.join(","),
            // ignore_path_str,
            self.optional.throttle.unwrap_or_default(),
            csv_file_str
        )
    }
}
