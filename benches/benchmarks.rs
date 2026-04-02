/*
 * SPDX-FileCopyrightText: 2019 - 2022 Armin Becher <becherarmin@gmail.com>
 * SPDX-FileCopyrightText: 2023 - 2026 Robin Vobruba <hoijui.quaero@gmail.com>
 *
 * SPDX-License-Identifier: MIT
 */

#[cfg(test)]
#[macro_use]
extern crate criterion;

use std::path::{Path, PathBuf};

use criterion::Criterion;
use mlc::config::{Config, OptionalConfig};
use mle::markup;

fn benches_dir() -> PathBuf {
    Path::new(file!()).parent().unwrap().into()
}

// fn init() {
//     let _ = env_logger::builder().is_test(true).try_init();
// }

async fn end_to_end_benchmark() {
    // init();
    let markup_types = vec![markup::Type::Markdown];
    let root = benches_dir().join("benchmark/markdown/ignore_me_dir");
    let ignore_paths = vec![];
    let markup_files = markup::Type::find(root.as_path().into(), markup_types, ignore_paths)
        .await
        .unwrap();
    let config = Config::new(
        mle::Config {
            markup_files,
            links: true,
            anchors: true,
            ignore_links: vec![],
        },
        OptionalConfig {
            debug: Some(true),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    mlc::run(&config).await.unwrap();
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("End to end benchmark", |b| b.iter(end_to_end_benchmark));
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = criterion_benchmark
}
criterion_main!(benches);
