use std::path::PathBuf;

use criterion::{criterion_group, criterion_main, Criterion};

fn build_federalist(c: &mut Criterion) {
    let path = PathBuf::from("./test/federalist-config/federalist.toml");
    let config = stork_search::config::Config::from_file(path).unwrap();

    c.bench_function("build::federalist", |b| {
        b.iter(|| stork_search::build(&config).unwrap())
    });
}

fn search_federalist_for_liberty(c: &mut Criterion) {
    let path = PathBuf::from("./test/federalist-config/federalist.toml");
    let config = stork_search::config::Config::from_file(path).unwrap();
    let index = stork_search::build(&config).unwrap();

    c.bench_function("search::federalist::liberty", |b| {
        b.iter(|| stork_search::search_with_index(&index, "liberty"))
    });
}

criterion_group!(benches, build_federalist, search_federalist_for_liberty);
criterion_main!(benches);
