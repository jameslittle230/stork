extern crate stork;

use std::env;
use std::fs;

use std::fs::File;
use std::io::BufReader;
use std::io::Read;

use stork::build_index;
use stork::search;
use stork::Config;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        // print_help();
    }

    let command = &args[1];

    if command == "--build" {
        let config = parse_config(std::path::PathBuf::from(&args[2]));
        build_index(config);
    }

    if command == "--search" {
        let file = File::open(&args[2]).unwrap();
        let mut buf_reader = BufReader::new(file);
        let mut index: Vec<u8> = Vec::new();
        let _bytes_read = buf_reader.read_to_end(&mut index);
        println!("{} bytes", index.len());
        search(&index, &args[3]);
    }
}

fn parse_config(config_filename: std::path::PathBuf) -> Config {
    let contents = fs::read_to_string(&config_filename).expect(&std::format!(
        "Something went wrong reading the file {}",
        &config_filename.to_str().unwrap()
    ));

    return toml::from_str(&contents).expect("Config file does not contain proper TOML syntax.");
}

// fn print_help() {}
