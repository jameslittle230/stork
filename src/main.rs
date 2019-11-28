use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::fmt;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::time::Instant;

#[derive(Deserialize)]
struct Config {
    scope: String,
    output: String,
}

// struct Entry {
//     path: std::path::PathBuf,
//     count: usize,
// }

// impl fmt::Display for Entry {
//     // This trait requires `fmt` with this exact signature.
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, "{}: {}", self.path.display(), self.count)
//     }
// }

// struct Field {}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_help();
    }

    let command = &args[1];
    // let argument = &args[2];
    // let debug = &args[3];

    if command == "--build" {
        build_index(std::path::PathBuf::from(&args[2]));
    }

    if command == "--search" {
        search(std::path::PathBuf::from(&args[2]), &args[3]);
    }
}

fn build_index(config_filename: std::path::PathBuf) {
    let start_time = Instant::now();
    let contents = fs::read_to_string(&config_filename).expect(&std::format!(
        "Something went wrong reading the file {}",
        &config_filename.to_str().unwrap()
    ));

    let config: Config =
        toml::from_str(&contents).expect("Config file does not contain proper TOML syntax.");

    let mut output: HashMap<String, HashMap<std::path::PathBuf, usize>> = HashMap::new();
    let filepaths = fs::read_dir(config.scope).unwrap();
    let mut filepath_count = 0;
    for path in filepaths {
        filepath_count += 1;
        let pathname = &path.unwrap().path();

        // For each word
        let file = File::open(&pathname).unwrap();
        let mut buf_reader = BufReader::new(file);
        let mut contents = String::new();
        let _bytes_read = buf_reader.read_to_string(&mut contents);

        let words_in_file: Vec<String> =
            contents.split_whitespace().map(|w| w.to_string()).collect();
        for word in words_in_file {
            let normalized_word = word.to_lowercase();
            let word_map = output.entry(normalized_word).or_insert(HashMap::new());
            let count = word_map.entry(pathname.to_path_buf()).or_insert(0);
            *count += 1;
        }
    }

    let output_file = File::create(&config.output).unwrap();
    serde_cbor::to_writer(&output_file, &output).unwrap();

    let elapsed = start_time.elapsed();
    println!(
        "Read {} files and wrote index at `{}` ({} bytes) in {} µs ({} s).",
        filepath_count,
        &config.output,
        &output_file.metadata().unwrap().len(),
        elapsed.as_micros(),
        elapsed.as_secs()
    )
}

fn search(index: std::path::PathBuf, query: &String) {
    let start_time = Instant::now();

    let file = File::open(&index).unwrap();
    let mut search_structure: HashMap<String, HashMap<std::path::PathBuf, usize>> =
        serde_cbor::from_reader(file).unwrap();
    let normalized_word = query.to_lowercase();
    println!("{:?}", &search_structure.entry(normalized_word));
    let elapsed = start_time.elapsed();
    println!("Found results in {} µs", elapsed.as_micros())
}

fn print_help() {}
