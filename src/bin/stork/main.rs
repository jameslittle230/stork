extern crate stork_search as stork;

mod argparse;
use argparse::Argparse;

mod test_server;
use test_server::serve;

mod display_timings;
use display_timings::*;

use std::fs::File;
use std::io::{BufReader, Read};
use std::time::Instant;
use std::{env, error::Error};
use stork::config::Config;
use stork::LatestVersion::structs::Index;

use num_format::{Locale, ToFormattedString};

pub type ExitCode = i32;
pub const EXIT_SUCCESS: ExitCode = 0;
pub const EXIT_FAILURE: ExitCode = 1;

fn help_text() -> String {
    return format!(
        r#"
Stork {}  --  by James Little
https://stork-search.net
Impossibly fast web search, made for static sites.

USAGE:
    stork --build [config.toml]

        Builds a search index from the specifications in the TOML configuration
        file. See https://stork-search.net/docs/build for more information.

    stork --test [config.toml]

        Builds a search index from the TOML configuration, then serves a test
        webpage on http://127.0.0.1:1612 that shows a search bar using that index.

    stork --search [./index.st] "[query]"

        Given a search index file, searches for the given query and outputs
        the results in JSON.
"#,
        env!("CARGO_PKG_VERSION")
    );
}

fn main() {
    let mut a = Argparse::new();
    a.register("build", build_handler, 1);
    a.register("test", test_handler, 1);
    a.register("search", search_handler, 2);
    a.register_help(&help_text());
    std::process::exit(a.exec(env::args().collect()));
}

pub fn build_index(config_path: &str) -> Result<(Config, Index), Box<dyn Error>> {
    let config = Config::from_file(std::path::PathBuf::from(config_path))?;
    let index = stork::build(&config)?;
    Ok((config, index))
}

fn build_handler(args: &[String]) {
    let start_time = Instant::now();

    let (config, index) = build_index(&args[2]).unwrap_or_else(|e| {
        eprintln!("Could not generate index: {}", e.to_string());
        std::process::exit(EXIT_FAILURE);
    });

    let build_time = Instant::now();
    let bytes_written = match index.write(&config) {
        Ok(bytes) => bytes,
        Err(e) => {
            eprintln!("Could not generate index: {}", e.to_string());
            std::process::exit(EXIT_FAILURE);
        }
    };

    let end_time = Instant::now();
    let bytes_per_file_string = format!(
        "{} bytes/entry (average entry size is {} bytes)",
        (bytes_written / index.entries_len()).to_formatted_string(&Locale::en),
        index.avg_entry_size().to_formatted_string(&Locale::en)
    );

    println!(
        "Index built, {} bytes written to {}.\n{}\n{}",
        bytes_written.to_formatted_string(&Locale::en),
        config.output.filename,
        {
            match bytes_written {
                0 => "(Maybe you're in debug mode.)",
                _ => bytes_per_file_string.as_str(),
            }
        },
        display_timings![
            (build_time.duration_since(start_time), "to build index"),
            (end_time.duration_since(build_time), "to write file"),
            (end_time.duration_since(start_time), "total")
        ]
    );
}

fn test_handler(args: &[String]) {
    let (_, index) = build_index(&args[2]).unwrap_or_else(|e| {
        eprintln!("Could not generate index: {}", e.to_string());
        std::process::exit(EXIT_FAILURE);
    });

    let _r = serve(index);
}

fn search_handler(args: &[String]) {
    let start_time = Instant::now();
    let file = File::open(&args[2]).unwrap_or_else(|err| {
        eprintln!("Could not read file {}: {}", &args[2], err);
        std::process::exit(EXIT_FAILURE);
    });

    let mut buf_reader = BufReader::new(file);
    let mut index_bytes: Vec<u8> = Vec::new();
    let bytes_read = buf_reader.read_to_end(&mut index_bytes);
    let read_time = Instant::now();

    match stork::parse_and_cache_index(&index_bytes, "a") {
        Ok(_info) => {}
        Err(e) => {
            eprintln!("Error parsing index: {}", e);
            std::process::exit(EXIT_FAILURE);
        }
    };

    let results = stork::search_from_cache("a", &args[3]);
    let end_time = Instant::now();

    match results {
        Ok(output) => {
            println!("{}", serde_json::to_string_pretty(&output).unwrap());

            eprintln!(
                "\n{} search results.\nRead {} bytes from {}\n{}",
                output.total_hit_count,
                bytes_read.unwrap().to_formatted_string(&Locale::en),
                &args[2],
                display_timings![
                    (read_time.duration_since(start_time), "to read index file"),
                    (end_time.duration_since(read_time), "to get search results"),
                    (end_time.duration_since(start_time), "total")
                ]
            );
        }
        Err(e) => eprintln!("Error performing search: {}", e),
    }
}
