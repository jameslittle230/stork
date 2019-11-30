use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::time::Instant;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
    fn alert(s: &str);
}

#[derive(Deserialize)]
struct Config {
    scope: String,
    output: String,
}

#[derive(Serialize)]
pub struct StorkResult {
    path: String,
    count: usize,
}

// impl fmt::Display for Entry {
//     // This trait requires `fmt` with this exact signature.
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, "{}: {}", self.path.display(), self.count)
//     }
// }

// struct Field {}

// #[cfg(not(target_arch = "wasm32"))]
fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        // print_help();
    }

    let command = &args[1];

    if command == "--build" {
        build_index(std::path::PathBuf::from(&args[2]));
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

pub fn search(index: &[u8], query: &String) -> Vec<StorkResult> {
    // let start_time = Instant::now();
    let search_structure_result: std::result::Result<
        HashMap<String, HashMap<std::path::PathBuf, usize>>,
        serde_cbor::error::Error,
    > = serde_cbor::from_slice(index);

    if (&search_structure_result).is_err() {
        println!("Failure to parse");
        let logged = format!("Failure to parse {} bytes", &index.len());
        log(&logged.as_str());
        panic!();
    }

    let search_structure = &mut search_structure_result.unwrap();
    let normalized_word = query.to_lowercase();
    let map = search_structure.get(&normalized_word).unwrap();
    let mut vec = Vec::new();
    map.iter().for_each(|e| {
        vec.push(StorkResult {
            count: *e.1,
            path: e.0.to_str().unwrap().to_string(),
        })
    });
    vec.sort_by(|a, b| b.count.cmp(&a.count));
    return vec;
}

// fn print_help() {}
