use std::{convert::TryFrom, path::PathBuf, time::Duration};
use stork_config::{Config, ConfigReadError};

use criterion::{criterion_group, criterion_main, Criterion};

// impl TryFrom<&PathBuf> for Config {
//     type Error = ConfigReadError;

//     fn try_from(value: &PathBuf) -> Result<Self, Self::Error> {
//         let contents = std::fs::read_to_string(value).unwrap();
//         Config::try_from(contents.as_str())
//     }
// }

fn build_federalist(c: &mut Criterion) {
    let path = PathBuf::from("./test/federalist-config/federalist.toml");
    let config = Config::try_from(&path).unwrap();

    let mut group = c.benchmark_group("build");
    group.measurement_time(Duration::from_secs(12));

    group.bench_function("federalist", |b| {
        b.iter(|| stork_lib::build(&config).unwrap())
    });
}

fn search_federalist_for_liberty(c: &mut Criterion) {
    let path = PathBuf::from("./test/federalist-config/federalist.toml");
    let config = Config::try_from(&path).unwrap();
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
