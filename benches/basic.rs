use std::{path::PathBuf, time::Duration};

use criterion::{criterion_group, criterion_main, Criterion};

fn build_federalist(c: &mut Criterion) {
    let path = PathBuf::from("./benches/federalist.toml");
    let config = stork_search::config::Config::from_file(path).unwrap();

    let mut group = c.benchmark_group("build");
    group.measurement_time(Duration::from_secs(12));

    group.bench_function("federalist", |b| {
        b.iter(|| stork_search::build(&config).unwrap())
    });
}

fn search_federalist_for_liberty(c: &mut Criterion) {
    let path = PathBuf::from("./benches/federalist.toml");
    let config = stork_search::config::Config::from_file(path).unwrap();
    let index = stork_search::build(&config).unwrap();

    let mut group = c.benchmark_group("search/federalist");
    group.measurement_time(Duration::from_secs(10));

    group.bench_function("liberty", |b| {
        b.iter(|| stork_search::search_with_index(&index, "liberty"))
    });

    group.bench_function("lib", |b| {
        b.iter(|| stork_search::search_with_index(&index, "lib"))
    });

    group.bench_function("liber old world", |b| {
        b.iter(|| stork_search::search_with_index(&index, "lib"))
    });
}

criterion_group!(benches, build_federalist, search_federalist_for_liberty);
criterion_main!(benches);
