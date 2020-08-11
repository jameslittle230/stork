extern crate stork_search as stork;

mod argparse;

use argparse::Argparse;
use std::env;
use std::fs::File;
use std::io::{BufReader, Read};
use std::time::Instant;
use stork::config::Config;

use num_format::{Locale, ToFormattedString};

const HELP_TEXT: &str = r#"
Stork 0.7.2  --  by James Little
https://stork-search.net
Impossibly fast web search, made for static sites.

USAGE:
    stork --build [config.toml]
    stork --search [./index.st] "[query]"
"#;

pub type ExitCode = i32;
pub const EXIT_SUCCESS: ExitCode = 0;
pub const EXIT_FAILURE: ExitCode = 1;

fn main() {
    let mut a = Argparse::new();
    a.register("build", build_handler, 1);
    a.register("search", search_handler, 2);
    a.register_help(HELP_TEXT);
    std::process::exit(a.exec(env::args().collect()));
}

fn build_handler(args: &[String]) {
    let start_time = Instant::now();
    let config = Config::from_file(std::path::PathBuf::from(&args[2]));
    let index = stork::build(&config).unwrap_or_else(|e| {
        eprintln!("Could not generate index: {}", e.to_string());
        std::process::exit(EXIT_FAILURE);
    });

    let build_time = Instant::now();
    let bytes_written = index.write(&config);
    let end_time = Instant::now();
    println!(
        "Index built, {} bytes written to {}. {}\n\t{:.3?}s to build index\n\t{:.3?}s to write file\n\t{:.3?}s total",
        bytes_written.to_formatted_string(&Locale::en),
        config.output.filename,
        {
            if bytes_written != 0 {
                ""
            } else {
                "(Maybe you're in debug mode.)"
            }
        },
        build_time.duration_since(start_time).as_secs_f32(),
        end_time.duration_since(build_time).as_secs_f32(),
        end_time.duration_since(start_time).as_secs_f32()
    );
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
    let results = stork::search(&index_bytes, args[3].clone());
    let end_time = Instant::now();

    match results {
        Ok(output) => {
            println!("{}", serde_json::to_string_pretty(&output).unwrap());

            eprintln!(
                "{} search results.\nRead {} bytes from {}\n\t{:.3?}s to read index file\n\t{:.3?}s to get search results\n\t{:.3?}s total",
                output.total_hit_count,
                bytes_read.unwrap().to_formatted_string(&Locale::en),
                &args[2],
                read_time.duration_since(start_time).as_secs_f32(),
                end_time.duration_since(read_time).as_secs_f32(),
                end_time.duration_since(start_time).as_secs_f32()
            );
        }
        Err(e) => eprintln!("Error performing search: {}", e),
    }
}
