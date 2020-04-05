use super::structs::*;
use crate::config::Config;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

extern crate rust_stemmers;
use rust_stemmers::{Algorithm, Stemmer};

pub fn build(config: &Config) -> Index {
    let en_stemmer = Stemmer::create(Algorithm::English);
    let mut entries: Vec<Entry> = Vec::new();
    let mut containers: HashMap<String, Container> = HashMap::new();

    let config = &config.input;

    // Step 1: Fill entries vector
    let base_directory = Path::new(&config.base_directory);
    for stork_file in config.files.as_ref().unwrap_or(&vec![]).iter() {
        let full_pathname = &base_directory.join(&stork_file.path);
        let file = File::open(&full_pathname).unwrap();
        let mut buf_reader = BufReader::new(file);
        let mut contents = String::new();
        let _bytes_read = buf_reader.read_to_string(&mut contents);
        let stork_fields = stork_file.fields.clone();

        let entry = Entry {
            contents,
            title: stork_file.title.clone(),
            url: stork_file.url.clone(),
            fields: stork_fields,
        };

        entries.push(entry);
    }

    let mut stems: HashMap<String, Vec<String>> = HashMap::new();

    for entry in &entries {
        let words: Vec<String> = entry
            .contents
            .split_whitespace()
            .map(|w| w.to_string())
            .collect();

        for word in words {
            let normalized_word = remove_surrounding_punctuation(&word.to_lowercase());
            let stem = en_stemmer.stem(&normalized_word).to_string();
            let stem_vector = stems.entry(stem).or_insert(vec![]);
            if !stem_vector.contains(&normalized_word) {
                stem_vector.push(normalized_word);
            }
        }
    }

    // Step 2: Fill containers map
    for (entry_index, entry) in entries.iter().enumerate() {
        let words_in_contents: Vec<String> = entry
            .contents
            .split_whitespace()
            .map(|w| w.to_string())
            .collect();

        for (word_index, word) in words_in_contents.iter().enumerate() {
            let normalized_word = remove_surrounding_punctuation(&word.to_lowercase());
            let normalized_word_len = &normalized_word.len();

            // Step 2A: Fill the container's results map
            let results_map = &mut containers
                .entry(normalized_word.clone())
                .or_insert_with(Container::new)
                .results;

            let entry_result: &mut SearchResult = results_map
                .entry(entry_index)
                .or_insert_with(SearchResult::new);

            entry_result.excerpts.push(Excerpt { word_index });

            // Step 2B: Fill _other containers'_ aliases maps with the
            // prefixes of this word
            for n in 3..*normalized_word_len {
                let substring = &normalized_word.as_str()[0..n].to_string();

                let alises_map = &mut containers
                    .entry(substring.clone())
                    .or_insert_with(Container::new)
                    .aliases;

                let _alias_score = alises_map
                    .entry(normalized_word.clone())
                    .or_insert(127 - (*normalized_word_len - n) as u8);
            }

            // Step 2C: Fill _other containers'_ alias maps with the
            // reverse-stems of this word
            let stem = en_stemmer.stem(&normalized_word).to_string();
            if let Some(reverse_stems_vector) = stems.get(&stem) {
                for reverse_stem in reverse_stems_vector {
                    if reverse_stem != &normalized_word {
                        let _alias_score = containers
                            .entry(reverse_stem.clone())
                            .or_insert_with(Container::new)
                            .aliases
                            .entry(normalized_word.clone())
                            .or_insert(50 as u8);
                    }
                }
            }
        }
    }

    Index {
        entries,
        queries: containers,
    }
}

fn remove_surrounding_punctuation(input: &str) -> String {
    let mut chars: Vec<char> = input.chars().collect();

    while chars.first().unwrap().is_ascii_punctuation() {
        chars.remove(0);
    }

    while chars.last().unwrap().is_ascii_punctuation() {
        chars.pop();
    }

    chars.into_iter().collect()
}
