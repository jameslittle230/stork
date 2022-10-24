use criterion::{criterion_group, criterion_main, Criterion};
use std::{convert::TryFrom, path::PathBuf, process::exit, time::Duration};
use stork_lib::build_config::Config;

fn config_from_path(path: &str) -> Config {
    if !std::path::Path::join(&std::env::current_dir().unwrap(), ".stork-project-root").exists() {
        println!("To successfully run this benchmark, the working directory must be the Stork project root.\nIt looks like the working directory is {:?}.\nRunning `just bench` will do this automatically.", std::env::current_dir());
        exit(50);
    }

    let path = PathBuf::from(path);
    let contents = std::fs::read_to_string(path).unwrap();
    return Config::try_from(contents.as_str()).unwrap();
}

fn build_federalist(c: &mut Criterion) {
    let config = config_from_path("./stork-lib/benches/federalist.toml");

    let mut group = c.benchmark_group("build");
    group.measurement_time(Duration::from_secs(12));

    group.bench_function("federalist", |b| {
        b.iter(|| stork_lib::build_index(&config, None).unwrap())
    });
}

fn search_federalist_for_liberty(c: &mut Criterion) {
    let config = config_from_path("./stork-lib/benches/federalist.toml");
    let bytes = stork_lib::build_index(&config, None).unwrap().primary_data;
    let index = stork_lib::parse_bytes_as_index(bytes).unwrap();

    let mut group = c.benchmark_group("search/federalist");
    group.measurement_time(Duration::from_secs(10));

    let queries = vec![
        "liberty",
        "lib",
        "liber old world",
        "some long query that won't return results but let's see how it does",
        "the old world", // "the" has about 4k search results
    ];

    for query in &queries {
        let mut id = query.to_owned().replace(' ', "-");
        id.truncate(12);
        group.bench_function(id, |b| {
            b.iter(|| stork_lib::search(&index, query.to_owned()))
        });
    }
}

criterion_group!(benches, build_federalist, search_federalist_for_liberty);
criterion_main!(benches);
