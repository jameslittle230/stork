use console_error_panic_hook;
use serde::Deserialize;
use std::cmp;
use std::collections::HashMap;
use std::path::Path;
use wasm_bindgen::prelude::*;

use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::time::Instant;

mod models;
use models::StorkEntry;
use models::StorkExcerpt;
use models::StorkResult;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn my_search(index: &[u8], query: String) -> String {
    console_error_panic_hook::set_once();
    return serde_json::to_string(&search(&index, &query)).unwrap();
}

#[derive(Deserialize)]
pub struct Config {
    input: ConfigInput,
    output: ConfigOutput,
}

#[derive(Deserialize)]
struct ConfigInput {
    surrounding_word_count: Option<u8>,
    base_directory: String,
    files: Vec<StorkEntry>,
}

#[derive(Deserialize)]
struct ConfigOutput {
    filename: String,
}

pub fn remove_surrounding_punctuation(input: &String) -> String {
    let mut chars: Vec<char> = input.chars().collect();

    while chars.first().unwrap().is_ascii_punctuation() {
        chars.remove(0);
    }

    while chars.last().unwrap().is_ascii_punctuation() {
        chars.pop();
    }

    return chars.into_iter().collect();
}

pub fn build_index(config: Config) -> (Vec<StorkEntry>, HashMap<String, Vec<StorkResult>>) {
    let start_time = Instant::now();

    let mut entries: Vec<StorkEntry> = Vec::new();
    let mut output: HashMap<String, Vec<StorkResult>> = HashMap::new();

    let base_directory = Path::new(&config.input.base_directory);
    for (index, entry_value) in config.input.files.iter().enumerate() {
        entries.push(entry_value.clone());
        let full_pathname = &base_directory.join(&entry_value.path);

        let file = File::open(&full_pathname).unwrap();
        let mut buf_reader = BufReader::new(file);
        let mut contents = String::new();
        let _bytes_read = buf_reader.read_to_string(&mut contents);

        let words_in_file: Vec<String> =
            contents.split_whitespace().map(|w| w.to_string()).collect();

        for (word_index, word) in words_in_file.iter().enumerate() {
            let normalized_word = remove_surrounding_punctuation(&word.to_lowercase());
            if normalized_word.len() < 3 {
                continue;
            }

            for n in 3..=normalized_word.len() {
                let substring = &normalized_word.as_str()[0..n].to_string();

                let range_width = config.input.surrounding_word_count.unwrap_or(8) as usize;
                let min_range = word_index.checked_sub(range_width).unwrap_or(0);
                let max_range = cmp::min(word_index + range_width, words_in_file.len() - 1);

                let excerpt = words_in_file[min_range..max_range].join(" ");
                let excerpt_iter = excerpt.chars();
                let mut offset = 0;

                while offset < excerpt.len() - normalized_word.len()
                    && excerpt_iter.as_str()[..normalized_word.len()] != normalized_word
                {
                    offset += 1;
                }

                let excerpt = StorkExcerpt {
                    value: excerpt,
                    query_offset: offset as u16,
                };

                let stork_result_vector = output.entry(substring.to_string()).or_insert(Vec::new());
                let mut should_create_new_result = true;
                for result in stork_result_vector.iter_mut() {
                    if result.file_index == (index as u32) {
                        result.excerpts.push(excerpt.clone());
                        should_create_new_result = false
                    }
                }

                if should_create_new_result == true {
                    stork_result_vector.push(StorkResult {
                        excerpts: vec![excerpt],
                        file_index: index as u32,
                        score: 10,
                    })
                }
            }
        }
    }

    println!("{:?}", output.entry("liberty".to_string()));

    let elapsed = start_time.elapsed();
    println!(
        "Read {} files and built index in {} µs ({} s).",
        &config.input.files.len(),
        // &config.output.filename,
        // &output_file.metadata().unwrap().len(),
        elapsed.as_micros(),
        elapsed.as_secs()
    );

    return (entries, output);
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
