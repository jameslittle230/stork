use serde::{Deserialize, Serialize};
use std::cmp;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::path::Path;
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
    input: ConfigInput,
    output: ConfigOutput,
}

#[derive(Deserialize)]
struct ConfigInput {
    base_directory: String,
    files: Vec<StorkEntry>,
}

#[derive(Deserialize)]
struct ConfigOutput {
    filename: String,
}

#[derive(Clone, Deserialize)]
struct StorkEntry {
    path: String,
    url: String,
    title: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StorkResult {
    url: String,
    excerpt: String,
    title: String,
}

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

fn remove_surrounding_punctuation(input: &String) -> String {
    let mut chars: Vec<char> = input.chars().collect();

    while chars.first().unwrap().is_ascii_punctuation() {
        chars.remove(0);
    }

    while chars.last().unwrap().is_ascii_punctuation() {
        chars.pop();
    }

    return chars.into_iter().collect();
}

fn build_index(config_filename: std::path::PathBuf) {
    let start_time = Instant::now();
    let contents = fs::read_to_string(&config_filename).expect(&std::format!(
        "Something went wrong reading the file {}",
        &config_filename.to_str().unwrap()
    ));

    let config: Config =
        toml::from_str(&contents).expect("Config file does not contain proper TOML syntax.");

    let mut output: HashMap<String, Vec<StorkResult>> = HashMap::new();

    let base_directory = Path::new(&config.input.base_directory);
    for entry_value in &config.input.files {
        let full_pathname = &base_directory.join(&entry_value.path);

        let file = File::open(&full_pathname).unwrap();
        let mut buf_reader = BufReader::new(file);
        let mut contents = String::new();
        let _bytes_read = buf_reader.read_to_string(&mut contents);

        let words_in_file: Vec<String> =
            contents.split_whitespace().map(|w| w.to_string()).collect();

        for (index, word) in words_in_file.iter().enumerate() {
            let normalized_word = remove_surrounding_punctuation(&word.to_lowercase());
            if normalized_word.len() < 3 {
                continue;
            }

            for n in 3..=normalized_word.len() {
                let stem = &normalized_word.as_str()[0..n].to_string();

                let range_width = 8;
                let min_range = index.checked_sub(range_width).unwrap_or(0);
                let max_range = cmp::min(index + range_width, words_in_file.len() - 1);
                let excerpt = words_in_file[min_range..max_range].join(" ");

                let title_clone = entry_value.title.as_str();
                let url_clone = entry_value.url.as_str();

                let stem_vector = output.entry(stem.to_string()).or_insert(Vec::new());
                stem_vector.push(StorkResult {
                    title: title_clone.to_string(),
                    url: url_clone.to_string(),
                    excerpt: excerpt,
                })
            }
        }
    }

    let output_file = File::create(&config.output.filename).unwrap();
    serde_cbor::to_writer(&output_file, &output).unwrap();

    let elapsed = start_time.elapsed();
    println!(
        "Read {} files and wrote index at `{}` ({} bytes) in {} µs ({} s).",
        &config.input.files.len(),
        &config.output.filename,
        &output_file.metadata().unwrap().len(),
        elapsed.as_micros(),
        elapsed.as_secs()
    )
}

pub fn search(index: &[u8], query: &String) -> Vec<StorkResult> {
    // let start_time = Instant::now();
    let search_structure_result: std::result::Result<
        HashMap<String, Vec<StorkResult>>,
        serde_cbor::error::Error,
    > = serde_cbor::from_slice(index);

    if (&search_structure_result).is_err() {
        println!("Failure to parse");
        panic!();
    }

    let search_structure = &mut search_structure_result.unwrap();
    let normalized_word = query.to_lowercase();

    let output: Vec<StorkResult> = search_structure
        .get(&normalized_word)
        .unwrap_or(&Vec::new())
        .to_owned()
        .to_vec();

    // println!(
    //     "Found {} results in {} µs",
    //     output.len(),
    //     start_time.elapsed().as_micros()
    // );

    println!("{:?}", output);
    return output;
}

// fn print_help() {}
