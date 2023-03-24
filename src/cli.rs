use crate::ignore_path::IgnorePath;
use crate::markup::MarkupType;
use crate::Config;
use crate::OptionalConfig;
use clap::Arg;
use clap::ArgAction;
use clap::Command;
use std::convert::TryFrom;
use std::fs;
use std::path::Path;
use std::path::MAIN_SEPARATOR;

const CONFIG_FILE_PATH: &str = "./.mlc.toml";

#[must_use]
pub fn parse_args() -> Config {
    let mut opt: OptionalConfig = match fs::read_to_string(CONFIG_FILE_PATH) {
        Ok(content) => match toml::from_str(&content) {
            Ok(o) => o,
            Err(err) => panic!("Invalid TOML file {:?}", err),
        },
        Err(_) => OptionalConfig::default(),
    };

    let matches = Command::new(crate_name!())
        .arg(
            Arg::new("directory")
                .help("Check all links in given directory and subdirectory")
                .required(false)
                .index(1),
        )
        .arg(
            Arg::new("debug")
                .long("debug")
                .short('d')
                .help("Print debug information to console")
                .required(false),
        )
        .arg(
            Arg::new("offline")
                .long("offline")
                .short('o')
                .help("Do not check web links")
                .required(false),
        )
        .arg(
            Arg::new("match-file-extension")
                .long("match-file-extension")
                .short('e')
                .help("Do check for the exact file extension when searching for a file")
                .action(ArgAction::SetTrue)
                .required(false),
        )
        .arg(
            Arg::new("ignore-path")
                .long("ignore-path")
                .short('p')
                .help("List of files and directories which will not be checked; space separated")
                .long_help("One or more files or directories which will not be checked, separated by white-space.")
                .value_name("PATHS")
                .value_delimiter(',')
                .action(ArgAction::Append)
                .required(false)
        )
        .arg(
            Arg::new("ignore-links")
                .long("ignore-links")
                .short('i')
                .help("List of links which will not be checked; space separated")
                .long_help("One or more wildcard-patterns/globs, matching links which will not be checked, separated by white-space.")
                .value_name("GLOBS")
                .value_delimiter(',')
                .action(ArgAction::Append)
                .required(false)
        )
        .arg(
            Arg::new("markup-types")
                .long("markup-types")
                .short('t')
                .value_name("TYPES")
                .help("List of markup types which shall be checked; space separated")
                .long_help("One or more markup file types which shall be checked, separated by white-space.")
                .action(ArgAction::Append)
                .value_delimiter(',')
                .value_parser(["md", "html"])
                .required(false)
        )
        .arg(
            Arg::new("throttle")
                .long("throttle")
                .short('T')
                .num_args(1)
                .value_name("DELAY_MS")
                .help("Wait between http request to the same host for a defined number of milliseconds")
                .required(false)
        )
        .arg(
            Arg::new("root-dir")
                .long("root-dir")
                .short('r')
                .num_args(1)
                .value_name("DIR")
                .help("Path to the root folder used to resolve all relative paths")
                .required(false)
        )
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .get_matches();

    let default_dir = format!(".{}", &MAIN_SEPARATOR);
    let dir_string = matches
        .get_one::<String>("directory")
        .unwrap_or(&default_dir);
    let directory = dir_string
        .replace('/', &MAIN_SEPARATOR.to_string())
        .replace('\\', &MAIN_SEPARATOR.to_string())
        .parse()
        .expect("failed to parse path");

    if matches.get_flag("debug") {
        opt.debug = Some(true);
    }

    if let Some(throttle) = matches.get_one::<u32>("throttle") {
        opt.throttle = Some(*throttle);
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

    if matches.get_flag("offline") {
        opt.offline = Some(true);
    }

    if matches.get_flag("match-file-extension") {
        opt.match_file_extension = Some(true)
    }

    if let Some(ignore_links) = matches.get_many::<String>("ignore-links") {
        opt.ignore_links = Some(ignore_links.map(|x| x.to_string()).collect());
    }

    if let Some(ignore_path) = matches.get_many::<String>("ignore-path") {
        opt.ignore_paths = Some(
            ignore_path
                .map(IgnorePath::try_from)
                .collect::<Result<Vec<IgnorePath>, _>>()
                .unwrap()
        );
    }

    if let Some(root_dir) = matches.get_one::<String>("root-dir") {
        let root_path = Path::new(
            &root_dir
                .replace('/', &MAIN_SEPARATOR.to_string())
                .replace('\\', &MAIN_SEPARATOR.to_string()),
        )
        .to_path_buf();
        if !root_path.is_dir() {
            eprintln!("Root path '{:?}' must be a directory!", root_path);
            std::process::exit(1);
        }
        opt.root_dir = Some(root_path)
    }

    Config {
        directory,
        optional: opt,
    }
}
