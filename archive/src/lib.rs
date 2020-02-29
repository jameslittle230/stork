use console_error_panic_hook;
use std::cmp;
use std::collections::HashMap;
use std::path::Path;

use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::io::{Read, Write};
use std::iter::FromIterator;

pub mod models;
use models::config::{ConfigInput, ConfigOutput};
use models::{StorkEntry, StorkExcerpt, StorkIndex, StorkOutput, StorkResult, StorkResultOrAlias};

mod utils;
use utils::remove_surrounding_punctuation;

mod index_helpers;
use index_helpers::{get_index_entries, get_index_results, get_index_version};

use wasm_bindgen::prelude::*;

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
pub fn search(index: &[u8], query: String) -> String {
    console_error_panic_hook::set_once();
    return serde_json::to_string(&perform_search(&index, &query)).unwrap();
}

#[wasm_bindgen]
pub fn extract_index_version(index: &[u8]) -> String {
    console_error_panic_hook::set_once();
    return serde_json::to_string(&get_index_version(&index)).unwrap();
}

pub fn build_index(config: &ConfigInput) -> StorkIndex {
    let mut entries: Vec<StorkEntry> = Vec::new();
    let mut output: HashMap<String, Vec<StorkResultOrAlias>> = HashMap::new();

    let base_directory = Path::new(&config.base_directory);
    for (index, entry_value) in config.files.iter().enumerate() {
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

            let range_width = config.surrounding_word_count.unwrap_or(8) as usize;
            let min_range = word_index.checked_sub(range_width).unwrap_or(0);
            let max_range = cmp::min(word_index + range_width, words_in_file.len() - 1);

            let excerpt = words_in_file[min_range..max_range].join(" ");

            // Adding 1 to account for the space after the range of words:
            // "foo bar baz" vs. "foo bar baz "
            let offset = words_in_file[min_range..word_index].join(" ").len() + 1;

            let excerpt = StorkExcerpt {
                value: excerpt,
                query_offset: offset as u16,
            };

            let stork_result_vector = output.entry(normalized_word.clone()).or_insert(Vec::new());
            let mut should_create_new_result = true;
            for result in stork_result_vector.iter_mut() {
                if let StorkResultOrAlias::Result(r) = result {
                    if r.file_index == (index as u32) {
                        r.excerpts.push(excerpt.clone());
                        should_create_new_result = false
                    }
                }
            }

            if should_create_new_result == true {
                stork_result_vector.push(StorkResultOrAlias::Result(StorkResult {
                    excerpts: vec![excerpt],
                    file_index: index as u32,
                    score: 10,
                }))
            }

            for n in 3..normalized_word.len() {
                let substring = &normalized_word.as_str()[0..n].to_string();
                let stork_result_vector = output.entry(substring.to_string()).or_insert(Vec::new());
                let mut should_create_new_alias = true;

                for result in stork_result_vector.iter_mut() {
                    if let StorkResultOrAlias::Alias(a) = result {
                        if a.to_string() == normalized_word {
                            should_create_new_alias = false;
                        }
                    }
                }

                if should_create_new_alias {
                    stork_result_vector.push(StorkResultOrAlias::Alias(normalized_word.clone()));
                }
            }
        }
    }

    return StorkIndex {
        entries: entries,
        results: output,
    };
}

pub fn write_index(config: &ConfigOutput, index: StorkIndex) -> usize {
    let file = File::create(&config.filename).unwrap();
    let mut bufwriter = BufWriter::new(file);

    let write_version = b"stork-1.0.0";
    if config.debug.unwrap_or(false) {
        let entries_encoded = serde_json::to_string_pretty(&index.entries).unwrap();
        let results_encoded = serde_json::to_string_pretty(&index.results).unwrap();
        let byte_vectors_to_write = [
            write_version,
            entries_encoded.as_bytes(),
            results_encoded.as_bytes(),
        ];

        for vec in byte_vectors_to_write.iter() {
            let _ = bufwriter.write(vec.len().to_string().as_bytes());
            let _ = bufwriter.write(b"\n");
            let _ = bufwriter.write(vec);
            let _ = bufwriter.write(b"\n\n");
        }

        return 0;
    } else {
        let mut bytes_written: usize = 0;

        let entries_encoded = bincode::serialize(&index.entries).unwrap();
        let results_encoded = bincode::serialize(&index.results).unwrap();
        let byte_vectors_to_write = [
            write_version,
            entries_encoded.as_slice(),
            results_encoded.as_slice(),
        ];

        for vec in byte_vectors_to_write.iter() {
            bytes_written += bufwriter.write(&(vec.len() as u64).to_be_bytes()).unwrap();
            bytes_written += bufwriter.write(vec).unwrap();
        }

        return bytes_written;
    }
}

pub fn perform_search(index: &[u8], query: &String) -> Vec<StorkOutput> {
    let normalized_query = query.to_lowercase();
    let mut words_in_query = normalized_query.split(" "); // not sure this needs to be mutable

    let mut first_results = perform_word_lookup(index, &words_in_query.next().unwrap().to_string());

    for query_word in words_in_query {
        for result in &mut first_results {
            result.excerpts = result
                .excerpts
                .iter()
                .filter(|e| e.value.contains(query_word))
                .cloned()
                .collect();
        }
    }

    first_results = first_results
        .iter()
        .filter(|&r| !r.excerpts.is_empty())
        .cloned()
        .collect();

    let entries = get_index_entries(index);
    let mut output_map: HashMap<usize, StorkOutput> = HashMap::new();
    for mut result in first_results {
        output_map
            .entry(result.file_index as usize)
            .and_modify(|e| e.result.excerpts.append(&mut result.excerpts))
            // and modify score, too
            .or_insert(StorkOutput {
                entry: entries[result.file_index as usize].clone(),
                result: result,
            });
    }

    let mut output_vector = Vec::from_iter(output_map.values().cloned());
    // eventually sort by score instead
    output_vector.sort_by_key(|o| o.result.file_index);
    return output_vector;
}

fn perform_word_lookup(index: &[u8], query: &String) -> Vec<StorkResult> {
    let version = get_index_version(index);
    let full_results = get_index_results(index);

    if version == "stork-1.0.0" {
        let query_result: Vec<StorkResultOrAlias> = full_results
            .get(query)
            .unwrap_or(&Vec::new())
            .to_owned()
            .to_vec();

        return expand_aliases_to_results(&full_results, &query_result);
        // At this point we should be able to guarantee that the results vector
        // is ordered by file index (but _not_ that there aren't file index
        // duplicates.)
    }

    // Throw error - unknown index version
    return vec![];
}

fn expand_aliases_to_results(
    full_results: &HashMap<String, Vec<StorkResultOrAlias>>,
    results_aliases: &Vec<StorkResultOrAlias>,
) -> Vec<StorkResult> {
    if results_aliases.len() == 0 {
        return vec![];
    }

    let mut output: Vec<StorkResult> = Vec::new();

    for sroa in results_aliases {
        if let StorkResultOrAlias::Result(r) = sroa {
            output.push(r.clone());
        } else if let StorkResultOrAlias::Alias(a) = sroa {
            let empty_vec = Vec::new();
            let alias_pointee = full_results.get(a).unwrap_or(&empty_vec);
            let expanded_inner_results = expand_aliases_to_results(full_results, alias_pointee);
            for inner_result in expanded_inner_results {
                output.push(inner_result);
            }
        }
    }

    return output;
}
