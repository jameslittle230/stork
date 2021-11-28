use criterion::{criterion_group, criterion_main, Criterion};
use std::{convert::TryFrom, path::PathBuf, time::Duration};
use stork_config::Config;

fn config_from_path(path: &str) -> Config {
    let path = PathBuf::from(path);
    let contents = std::fs::read_to_string(path).unwrap();
    return Config::try_from(contents.as_str()).unwrap();
}

fn build_federalist(c: &mut Criterion) {
    let config = config_from_path("./test-assets/federalist.toml");

    let mut group = c.benchmark_group("build");
    group.measurement_time(Duration::from_secs(12));

    group.bench_function("federalist", |b| {
        b.iter(|| stork_lib::V3Build(&config).unwrap())
    });
}

fn search_federalist_for_liberty(c: &mut Criterion) {
    let config = config_from_path("./test-assets/federalist.toml");
    let index = stork_lib::V3Build(&config).unwrap().index;

    let mut group = c.benchmark_group("search/federalist");
    group.measurement_time(Duration::from_secs(10));

    let queries = vec![
        "liberty",
        "lib",
        "liber old world",
        "some long query that won't return results but let's see how it does",
    ];

    for query in &queries {
        group.bench_function(query.to_owned(), |b| {
            b.iter(|| stork_lib::V3Search(&index, query.to_owned()))
        });
    }
}

criterion_group!(benches, build_federalist, search_federalist_for_liberty);
criterion_main!(benches);
