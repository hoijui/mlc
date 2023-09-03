#[cfg(test)]
mod helper;

use helper::benches_dir;
use mlc::markup::MarkupType;
use mlc::Config;
use mlc::OptionalConfig;
use std::fs;

#[tokio::test]
async fn end_to_end() {
    let config = Config {
        directory: benches_dir().join("benchmark"),
        optional: OptionalConfig {
            debug: None,
            markup_types: Some(vec![MarkupType::Markdown]),
            offline: None,
            match_file_extension: None,
            throttle: None,
            ignore_links: Some(vec!["./doc/broken-local-link.doc".to_string()]),
            ignore_paths: Some(vec![
                fs::canonicalize("benches/benchmark/markdown/ignore_me.md").unwrap(),
                fs::canonicalize("./benches/benchmark/markdown/ignore_me_dir").unwrap(),
            ]),
            root_dir: None,
        },
    };
    if let Err(e) = mlc::run(&config).await {
        panic!("Test with custom root failed. {:?}", e);
    }
}

#[tokio::test]
async fn end_to_end_different_root() {
    let test_files = benches_dir().join("different_root");
    let config = Config {
        directory: test_files.clone(),
        optional: OptionalConfig {
            debug: Some(true),
            markup_types: Some(vec![MarkupType::Markdown]),
            offline: None,
            match_file_extension: None,
            ignore_links: None,
            ignore_paths: None,
            throttle: None,
            root_dir: Some(test_files),
        },
    };
    if let Err(e) = mlc::run(&config).await {
        panic!("Test with custom root failed. {:?}", e);
    }
}
