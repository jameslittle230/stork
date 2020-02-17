extern crate stork;

mod argparse;

use argparse::Argparse;
use std::env;
use std::fs::File;
use std::io::{BufReader, Read};
use stork::config::Config;

const HELP_TEXT: &str = r#"
Stork 1.0.0  --  by James Little
https://stork-search.net
Impossibly fast web search, made for static sites.

USAGE:
    stork --build [config.toml]
    stork --search [index.st] "[query]"
"#;

fn main() {
    let mut a = Argparse::new();
    a.register("build", build_handler);
    a.register("search", search_handler);
    a.register_help(HELP_TEXT);
    a.exec(env::args().collect());
}

fn build_handler(args: &[String]) {
    let argslen = args.len() as u8;
    assert!(argslen == 3, "Wrong number of arguments.");
    let config = Config::from_file(std::path::PathBuf::from(&args[2]));
    let index = stork::build(&config);
    let bytes_written = stork::write_index(&config, index);
    println!("{} bytes written", bytes_written)
}

fn search_handler(args: &[String]) {
    let argslen = args.len() as u8;
    assert!(argslen == 4, "Wrong number of arguments.");

    let file = File::open(&args[2]).unwrap();
    let mut buf_reader = BufReader::new(file);
    let mut index_bytes: Vec<u8> = Vec::new();
    let bytes_read = buf_reader.read_to_end(&mut index_bytes);
    println!("{} bytes read", bytes_read.unwrap());
    let results = stork::search(&index_bytes, args[3].to_owned());
    println!("{}", serde_json::to_string_pretty(&results).unwrap());
}
