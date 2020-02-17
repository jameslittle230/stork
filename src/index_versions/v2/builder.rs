use super::structs::*;
use crate::config::Config;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

pub fn build(config: &Config) -> Index {
    let mut entries: Vec<Entry> = Vec::new();
    let mut queries: HashMap<String, Container> = HashMap::new();
    let config = &config.input;

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

    for (entry_index, entry) in entries.iter().enumerate() {
        let words_in_contents: Vec<String> = entry
            .contents
            .split_whitespace()
            .map(|w| w.to_string())
            .collect();

        for (word_index, word) in words_in_contents.iter().enumerate() {
            let normalized_word = remove_surrounding_punctuation(&word.to_lowercase());
            let normalized_word_len = &normalized_word.len();

            let entry_result: &mut SearchResult = queries
                .entry(normalized_word.clone())
                .or_insert_with(Container::new)
                .results
                .entry(entry_index)
                .or_insert_with(SearchResult::new);

            entry_result.excerpts.push(Excerpt {
                contents: RangeInText {
                    begin: word_index.saturating_sub(8),
                    end: word_index.saturating_add(8),
                },
                highlights: vec![RangeInText {
                    begin: word_index,
                    end: word_index + 1
                }]
            });

            for n in 3..*normalized_word_len {
                let substring = &normalized_word.as_str()[0..n].to_string();

                let _alias_score = queries
                    .entry(substring.clone())
                    .or_insert_with(Container::new)
                    .aliases
                    .entry(normalized_word.clone())
                    .or_insert(127 - (*normalized_word_len - n) as u8);
            }
        }
    }

    Index {
        entries: entries,
        queries: queries,
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
