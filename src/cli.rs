/*
 * SPDX-FileCopyrightText: 2019 - 2022 Armin Becher <becherarmin@gmail.com>
 * SPDX-FileCopyrightText: 2022 Gervasio Marchand <github@gervas.io>
 * SPDX-FileCopyrightText: 2022 - 2025 Robin Vobruba <hoijui.quaero@gmail.com>
 * SPDX-FileCopyrightText: 2023 Paul Hazen <paul-hazen@live.com>
 *
 * SPDX-License-Identifier: MIT
 */

use clap::Arg;
use clap::ArgAction;
use clap::ArgGroup;
use clap::Command;
use clap::command;
use cli_utils::path_buf::PathBuf;
use mlc::Config;
use mlc::OptionalConfig;
use mle::BoxResult;
use mle::markup::Type as MarkupType;
use std::fs;
use std::path::MAIN_SEPARATOR_STR;
use std::path::Path;
use std::str::FromStr;
use std::sync::LazyLock;
use wildmatch::WildMatch;

const CONFIG_FILE_PATH: &str = "./.mlc.toml";

pub const A_L_DEBUG: &str = "debug";
pub const A_S_DEBUG: char = 'd';
pub const A_L_OFFLINE: &str = "offline";
pub const A_S_OFFLINE: char = 'o';
pub const A_L_NO_REDIRECT_WARN: &str = "do-not-warn-for-redirect-to";
pub const A_L_THROTTLE: &str = "throttle";
pub const A_S_THROTTLE: char = 'T';
pub const A_L_TIMEOUT: &str = "timeout";
pub const A_L_ROOT_DIR: &str = "root-dir";
pub const A_S_ROOT_DIR: char = 'r';

pub fn arg_debug() -> Arg {
    Arg::new(A_L_DEBUG)
        .long(A_L_DEBUG)
        .short(A_S_DEBUG)
        .help("Print debug information to console")
        .action(ArgAction::SetTrue)
}

pub fn arg_offline() -> Arg {
    Arg::new(A_L_OFFLINE)
        .long(A_L_OFFLINE)
        .short(A_S_OFFLINE)
        .help("Do not check web links")
        .action(ArgAction::SetTrue)
}

pub fn arg_no_redirect_warn() -> Arg {
    Arg::new(A_L_NO_REDIRECT_WARN)
        .long(A_L_NO_REDIRECT_WARN)
        .value_name("URL")
        .value_delimiter(',')
        .action(ArgAction::Append)
        .help("Comma separated list of links for which redirection will be ignored")
}

pub fn arg_throttle() -> Arg {
    Arg::new(A_L_THROTTLE)
        .long(A_L_THROTTLE)
        .short(A_S_THROTTLE)
        .num_args(1)
        .value_name("DELAY_MS")
        .help("Wait between http request to the same host for a defined number of milliseconds")
}

pub fn arg_timeout() -> Arg {
    Arg::new(A_L_TIMEOUT)
        .long(A_L_TIMEOUT)
        .num_args(1)
        .value_name("DELAY_MS")
        .help("Wait for HTTP request response for a maximum of milliseconds")
}

pub fn arg_root_dir() -> Arg {
    Arg::new(A_L_ROOT_DIR)
        .long(A_L_ROOT_DIR)
        .short(A_S_ROOT_DIR)
        .num_args(1)
        .value_name("DIR")
        .help("Path to the root folder used to resolve all relative paths")
    // .default_value(".")
}

static ARGS: LazyLock<[Arg; 12]> = LazyLock::new(|| {
    [
        mle::cli::arg_version(),
        mle::cli::arg_quiet(),
        arg_debug(),
        arg_offline(),
        arg_no_redirect_warn(),
        arg_throttle(),
        arg_timeout(),
        arg_root_dir(),
        mle::cli::arg_markup_files(),
        mle::cli::arg_markup_files_list(),
        mle::cli::arg_ignore_links(),
        mle::cli::arg_result_format(),
    ]
});

fn arg_matcher() -> Command {
    let duplicate_short_options = mle::cli::find_duplicate_short_options();
    assert!(
        duplicate_short_options.is_empty(),
        "Duplicate argument short options: {duplicate_short_options:?}",
    );
    command!()
        .bin_name(clap::crate_name!())
        .help_expected(true)
        .disable_version_flag(true)
        .args(ARGS.iter())
        .group(
            ArgGroup::new("markup-files")
                .args([mle::cli::A_N_MARKUP_FILES, mle::cli::A_L_MARKUP_FILES_LIST])
                .required(true),
        )
}

pub async fn parse_args() -> BoxResult<Config> {
    let mut opt: OptionalConfig = match fs::read_to_string(CONFIG_FILE_PATH) {
        Ok(content) => match toml::from_str(&content) {
            Ok(o) => o,
            Err(err) => panic!("Invalid TOML file {err:?}"),
        },
        Err(_) => OptionalConfig::default(),
    };

    let matches = arg_matcher().get_matches();

    let quiet = matches.get_flag(mle::cli::A_L_QUIET);
    let version = matches.get_flag(mle::cli::A_L_VERSION);
    if version {
        mle::cli::print_version_and_exit(mlc::VERSION, quiet);
    }

    let extractor_cfg = mle::cli::parse_args().await?;
    // let mut extractor_cfg = mle::config::Config::default();

    // let default_dir = format!(".{}", &MAIN_SEPARATOR);
    // let dir_string = matches
    //     .get_one::<String>("directory")
    //     .unwrap_or(&default_dir);
    // let directory: PathBuf = dir_string
    //     .replace(['/', '\\'], MAIN_SEPARATOR_STR)
    //     .parse()
    //     .expect("failed to parse path");

    // extractor_cfg.files_and_dirs.push(directory.clone());

    if matches.get_flag(A_L_DEBUG) {
        opt.debug = Some(true);
    }

    if let Some(do_not_warn_for_redirect_to) = matches.get_many::<String>(A_L_NO_REDIRECT_WARN) {
        opt.do_not_warn_for_redirect_to = Some(
            do_not_warn_for_redirect_to
                .map(|x| WildMatch::new(x))
                .collect(),
        );
    }

    if let Some(throttle_str) = matches.get_one::<String>(A_L_THROTTLE) {
        let throttle = throttle_str.parse::<u32>().unwrap();
        opt.throttle = Some(throttle);
    }

    if let Some(f) = matches.get_one::<String>("csv") {
        opt.csv_file = Some(
            Path::new(&f.replace(['/', '\\'], std::path::MAIN_SEPARATOR_STR))
                .to_path_buf()
                .into(),
        );
    }

    if let Some(markup_types) = matches.get_many::<String>("markup-types") {
        opt.markup_types = Some(
            markup_types
                .map(|v| v.as_str().parse().expect("invalid markup type"))
                .collect(),
        );
    }
    if opt.markup_types.is_none() {
        opt.markup_types = Some(vec![MarkupType::Markdown, MarkupType::Html]);
    }

    if matches.get_flag(A_L_OFFLINE) {
        opt.offline = Some(true);
    }

    if matches.get_flag("match-file-extension") {
        opt.match_file_extension = Some(true);
    }

    // if let Some(ignore_links) = matches.get_many::<String>("ignore-links") {
    //     // opt.ignore_links = Some(ignore_links.map(ToString::to_string).collect());
    //     extractor_cfg.ignore_links = ignore_links.map(|glob| WildMatch::new(glob)).collect();
    // }

    if let Some(root_dir) = matches.get_one::<String>(A_L_ROOT_DIR) {
        let root_path =
            PathBuf::from_str(root_dir.replace(['/', '\\'], MAIN_SEPARATOR_STR).as_str())
                .expect("Infallible");
        if !root_path.is_dir().await {
            eprintln!("Root path '{root_path:?}' must be a directory!");
            std::process::exit(1);
        }
        opt.root_dir = Some(root_path);
    }

    Config::new(extractor_cfg, opt).await
}
