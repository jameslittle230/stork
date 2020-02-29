extern crate stork;

use std::env;
use std::fs;

use std::fs::File;
use std::io::{BufReader, Read};

use stork::models::config::Config;
use stork::{build_index, perform_search, write_index};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_help();
        return;
    }

    let command = &args[1];

    if command == "--build" {
        let config = parse_config(std::path::PathBuf::from(&args[2]));
        let index = build_index(&config.input);
        let bytes_written = write_index(&config.output, index);
        println!("{} bytes written", bytes_written)
    }

    if command == "--search" {
        let file = File::open(&args[2]).unwrap();
        let mut buf_reader = BufReader::new(file);
        let mut index: Vec<u8> = Vec::new();
        let bytes_read = buf_reader.read_to_end(&mut index);
        println!("{} bytes read", bytes_read.unwrap());
        let results = perform_search(&index, &args[3]);
        println!("{}", serde_json::to_string_pretty(&results).unwrap());
    }
}

pub fn parse_config(config_filename: std::path::PathBuf) -> Config {
    let contents = fs::read_to_string(&config_filename).expect(&std::format!(
        "Something went wrong reading the file {}",
        &config_filename.to_str().unwrap()
    ));

    return toml::from_str(&contents).expect("Config file does not contain proper TOML syntax.");
}

fn print_help() {
    println!("");
    println!(
        "Stork 1.0.0  --  by James Little\nhttps://stork-search.net\n
Acceptable command line params:
--build [config.toml]
--search [config.toml] \"[query]\""
    );
    println!("");
}
