#[cfg(test)]
mod helper;

use clap::ValueEnum;
use helper::benches_dir;
use mlc::Config;
use mlc::OptionalConfig;
use mle::markup::Type as MarkupType;
use mle::path_buf::PathBuf;
use std::convert::TryInto;

// #[tokio::test]
async fn end_to_end() {
    let directory: PathBuf = benches_dir().join("benchmark").into();
    let config = Config::new(
        directory.clone(),
        mle::Config {
            files_and_dirs: vec![directory.clone()],
            recursive: true,
            links: Some(None),
            anchors: Some(None),
            ignore_paths: vec![],
            ignore_links: vec![],
            markup_types: MarkupType::value_variants().to_vec(),
            result_format: mle::result::Type::Markdown,
            result_extended: true,
            result_flush: true,
        },
        OptionalConfig {
            debug: None,
            markup_types: Some(vec![MarkupType::Markdown]),
            offline: None,
            match_file_extension: None,
            throttle: None,
            ignore_links: Some(vec!["./doc/broken-local-link.doc".to_string()]),
            ignore_paths: Some(vec![
                "benches/benchmark/markdown/ignore_me.md"
                    .try_into()
                    .unwrap(),
                "./benches/benchmark/markdown/ignore_me_dir"
                    .try_into()
                    .unwrap(),
            ]),
            root_dir: None,
        },
    )
    .await
    .unwrap();
    if let Err(err) = mlc::run(&config).await {
        panic!("Test with custom root failed. {err:?}");
    }
}

#[tokio::test]
async fn end_to_end_different_root() {
    let test_files = benches_dir().join("different_root");
    let config = Config::new(
        test_files.clone().into(),
        mle::Config {
            files_and_dirs: vec![test_files.clone().into()],
            recursive: true,
            links: Some(None),
            anchors: Some(None),
            ignore_paths: vec![],
            ignore_links: vec![],
            markup_types: MarkupType::value_variants().to_vec(),
            result_format: mle::result::Type::Markdown,
            result_extended: true,
            result_flush: true,
        },
        OptionalConfig {
            debug: Some(true),
            markup_types: Some(vec![MarkupType::Markdown]),
            offline: None,
            match_file_extension: None,
            ignore_links: None,
            ignore_paths: None,
            throttle: None,
            root_dir: Some(test_files.into()),
        },
    )
    .await
    .unwrap();
    if let Err(err) = mlc::run(&config).await {
        panic!("Test with custom root failed. {err:?}");
    }
}
