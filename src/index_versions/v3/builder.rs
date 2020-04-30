use super::scores::*;
use super::structs::*;
use super::word_list_generators::{
    HTMLWordListGenerator, PlainTextWordListGenerator,
    SRTWordListGenerator, WordListGenerator,
};
use crate::config::DataSource;
use crate::config::{Config, Filetype};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

extern crate rust_stemmers;
use rust_stemmers::{Algorithm, Stemmer};

extern crate htmlescape;
use htmlescape::encode_minimal;

pub(super) struct IntermediateEntry {
    pub(super) contents: Contents,
    pub(super) title: String,
    pub(super) url: String,
    pub(super) fields: Fields,
}

pub fn build(config: &Config) -> Index {
    let en_stemmer = Stemmer::create(Algorithm::English);
    let mut intermediate_entries: Vec<IntermediateEntry> = Vec::new();
    let mut containers: HashMap<String, Container> = HashMap::new();

    let config = &config.input;

    // Step 1: Fill entries vector
    let base_directory = Path::new(&config.base_directory);
    for stork_file in config.files.iter() {
        let filetype = &stork_file.computed_filetype().unwrap_or_else(|| panic!("Cannot determine a filetype for {}. Please include a filetype field in your config file or use a known file extension.", &stork_file.title));

        let buffer: String = match &stork_file.source {
            DataSource::Contents(contents) => contents.to_string(),
            DataSource::FilePath(path_string) => {
                let full_pathname = &base_directory.join(&path_string);
                let file = File::open(&full_pathname).unwrap();
                let mut buf_reader = BufReader::new(file);
                let mut c = String::new();
                let _bytes_read = buf_reader.read_to_string(&mut c);
                encode_minimal(&c)
            }
            DataSource::URL(_url) => panic!("URL not available yet"),
        };

        fn returns_word_list_generator(filetype: &Filetype) -> Box<dyn WordListGenerator> {
            match filetype {
                Filetype::PlainText => Box::new(PlainTextWordListGenerator {}),
                Filetype::SRTSubtitle => Box::new(SRTWordListGenerator {}),
                Filetype::HTML => Box::new(HTMLWordListGenerator {}),
            }
        }

        let contents: Contents = returns_word_list_generator(filetype).create_word_list(&buffer);

        let entry = IntermediateEntry {
            contents,
            title: stork_file.title.clone(),
            url: stork_file.url.clone(),
            fields: stork_file.fields.clone(),
        };

        intermediate_entries.push(entry);
    }

    let mut stems: HashMap<String, Vec<String>> = HashMap::new();

    for entry in &intermediate_entries {
        let contents = &entry.contents;

        for annotated_word in contents.word_list.iter() {
            let normalized_word =
                remove_surrounding_punctuation(&annotated_word.word.to_lowercase());
            let stem = en_stemmer.stem(&normalized_word).to_string();
            let stem_vector = stems.entry(stem).or_insert_with(|| vec![]);
            if !stem_vector.contains(&normalized_word) {
                stem_vector.push(normalized_word);
            }
        }
    }

    // Step 2: Fill containers map
    for (entry_index, entry) in intermediate_entries.iter().enumerate() {
        let words_in_contents: Vec<AnnotatedWord> = entry.contents.word_list.to_owned();

        for (word_index, annotated_word) in words_in_contents.iter().enumerate() {
            let normalized_word =
                remove_surrounding_punctuation(&annotated_word.word.to_lowercase());
            let normalized_word_len = &normalized_word.len();

            // Step 2A: Fill the container's results map
            let results_map = &mut containers
                .entry(normalized_word.clone())
                .or_insert_with(Container::new)
                .results;

            let entry_result: &mut SearchResult = results_map
                .entry(entry_index)
                .or_insert_with(SearchResult::new);

            entry_result.excerpts.push(Excerpt {
                word_index,
                ..Default::default()
            });

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
                    .or_insert(PREFIX_SCORE - (*normalized_word_len - n) as u8);
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
                            .or_insert(STEM_SCORE as u8);
                    }
                }
            }
        }
    }

    let entries: Vec<Entry> = intermediate_entries.iter().map(Entry::from).collect();

    Index {
        entries,
        containers,
        ..Default::default()
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
