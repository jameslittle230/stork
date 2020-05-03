extern crate stork_search as stork;

mod argparse;

use argparse::Argparse;
use std::env;
use std::fs::File;
use std::io::{BufReader, Read};
use std::time::Instant;
use stork::config::Config;
use stork::searcher::{search, SearchOutput};

use num_format::{Locale, ToFormattedString};

const HELP_TEXT: &str = r#"
Stork 1.0.0  --  by James Little
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
    let start = Instant::now();
    let config = Config::from_file(std::path::PathBuf::from(&args[2]));
    let index = stork::build(&config);
    let build = Instant::now();
    let bytes_written = stork::write_index(&config, index);
    let end = Instant::now();
    println!(
        "{} bytes written. {}
    {:.3?}s from start to build
    {:.3?}s to write file
    {:.3?}s total",
        bytes_written.to_formatted_string(&Locale::en),
        {
            if bytes_written != 0 {
                ""
            } else {
                "(Maybe you're in debug mode.)"
            }
        },
        build.duration_since(start).as_secs_f32(),
        end.duration_since(build).as_secs_f32(),
        end.duration_since(start).as_secs_f32()
    );
}

fn search_handler(args: &[String]) {
    let start = Instant::now();
    let file = File::open(&args[2]).unwrap();
    let mut buf_reader = BufReader::new(file);
    let mut index_bytes: Vec<u8> = Vec::new();
    let bytes_read = buf_reader.read_to_end(&mut index_bytes);
    let read = Instant::now();
    let results: SearchOutput = search(&index_bytes, &args[3]);
    let end = Instant::now();
    println!("{}", serde_json::to_string_pretty(&results).unwrap());

    println!(
        "read {} bytes.
    {:.3?}s to read index file
    {:.3?}s to get search results
    {:.3?}s total",
        bytes_read.unwrap().to_formatted_string(&Locale::en),
        read.duration_since(start).as_secs_f32(),
        end.duration_since(read).as_secs_f32(),
        end.duration_since(start).as_secs_f32()
    );
}
