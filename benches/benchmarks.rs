/*
 * SPDX-FileCopyrightText: 2019 - 2022 Armin Becher <becherarmin@gmail.com>
 * SPDX-FileCopyrightText: 2023 - 2025 Robin Vobruba <hoijui.quaero@gmail.com>
 *
 * SPDX-License-Identifier: MIT
 */

#[cfg(test)]
#[macro_use]
extern crate criterion;

use cli_utils::StreamIdent;
use criterion::Criterion;
use mlc::{Config, OptionalConfig};
use mle::markup::{self, Type as MarkupType};
use std::fs;

// fn init() {
//     let _ = env_logger::builder().is_test(true).try_init();
// }

async fn end_to_end_benchmark() {
    // init();
    let markup_types = vec![markup::Type::Markdown];
    let root = fs::canonicalize("./benches/benchmark/markdown/ignore_me_dir").unwrap();
    let ignore_paths = vec![];
    let markup_files = markup::Type::find(root.as_path().into(), markup_types, ignore_paths)
        .await
        .unwrap();
    let config = Config::new(
        mle::Config {
            markup_files,
            links: Some(StreamIdent::StdOut),
            anchors: Some(StreamIdent::StdOut),
            ignore_links: vec![],
            result_format: mle::result::Type::Markdown,
            result_extended: true,
            result_flush: true,
        },
        OptionalConfig {
            markup_types: Some(vec![MarkupType::Markdown]),
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
