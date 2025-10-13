#[cfg(test)]
#[macro_use]
extern crate criterion;

use clap::ValueEnum;
use criterion::Criterion;
use mlc::{Config, OptionalConfig};
use mle::{markup::Type as MarkupType, path_buf::PathBuf};
use std::fs;

async fn end_to_end_benchmark() {
    let directory: PathBuf = fs::canonicalize("./benches/benchmark/markdown/ignore_me_dir")
        .unwrap()
        .into();
    let config = Config::new(
        directory.clone(),
        mle::Config {
            files_and_dirs: vec![directory],
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
            markup_types: Some(vec![MarkupType::Markdown]),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    let _ = mlc::run(&config);
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
