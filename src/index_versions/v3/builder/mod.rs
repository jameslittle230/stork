use super::structs::{
    AnnotatedWord, Container, Entry, Excerpt, Index, PassthroughConfig, SearchResult,
    WordListSource,
};
use crate::config::Config;
use std::collections::HashMap;

mod fill_containers;
mod fill_intermediate_entries;
mod fill_stems;

mod annotated_words_from_string;
pub mod errors;
pub mod intermediate_entry;

use colored::Colorize;
use fill_containers::fill_containers;
use fill_intermediate_entries::fill_intermediate_entries;
use fill_stems::fill_stems;

use errors::{DocumentError, IndexGenerationError, WordListGenerationError};

use intermediate_entry::NormalizedEntry;

pub mod nudger;
use nudger::Nudger;

pub fn build(config: &Config) -> Result<(Index, Vec<DocumentError>), IndexGenerationError> {
    Nudger::from(config).print();

    let mut intermediate_entries: Vec<NormalizedEntry> = Vec::new();
    let mut document_errors: Vec<DocumentError> = Vec::new();
    fill_intermediate_entries(config, &mut intermediate_entries, &mut document_errors)?;

    if !document_errors.is_empty() {
        eprintln!(
            "{} {} error{} while indexing files. Your index was still generated, though the erroring files were omitted.",
            "Warning:".yellow(),
            document_errors.len(),
            match document_errors.len() {
                1 => "",
                _ => "s",
            }
        )
    }
    for error in &document_errors {
        eprintln!("{}", &error);
    }

    if intermediate_entries.is_empty() {
        return Err(IndexGenerationError::NoValidFiles);
    }

    let mut stems: HashMap<String, Vec<String>> = HashMap::new();
    fill_stems(&intermediate_entries, &mut stems);

    let mut containers: HashMap<String, Container> = HashMap::new();
    fill_containers(config, &intermediate_entries, &stems, &mut containers);

    let entries: Vec<Entry> = intermediate_entries.iter().map(Entry::from).collect();

    let config = PassthroughConfig {
        url_prefix: config.input.url_prefix.clone(),
        title_boost: config.input.title_boost.clone(),
        excerpt_buffer: config.output.excerpt_buffer,
        excerpts_per_result: config.output.excerpts_per_result,
        displayed_results_count: config.output.displayed_results_count,
    };

    Ok((
        Index {
            config,
            entries,
            containers,
        },
        document_errors,
    ))
}

fn remove_surrounding_punctuation(input: &str) -> String {
    let mut chars: Vec<char> = input.chars().collect();

    while chars.first().unwrap_or(&'a').is_ascii_punctuation() {
        chars.remove(0);
    }

    while chars.last().unwrap_or(&'a').is_ascii_punctuation() {
        chars.pop();
    }

    chars.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::File;
    use crate::config::{Config, DataSource, Filetype, InputConfig};

    fn generate_invalid_file_missing_selector() -> File {
        File {
            explicit_source: Some(DataSource::Contents("".to_string())),
            title: "Missing Selector".to_string(),
            filetype: Some(Filetype::HTML),
            html_selector_override: Some(".article".to_string()),
            ..Default::default()
        }
    }

    fn generate_invalid_file_empty_contents() -> File {
        File {
            explicit_source: Some(DataSource::Contents("".to_string())),
            title: "Empty Contents".to_string(),
            filetype: Some(Filetype::PlainText),
            ..Default::default()
        }
    }

    fn generate_valid_file() -> File {
        File {
            explicit_source: Some(DataSource::Contents("This is contents".to_string())),
            title: "Successful File".to_string(),
            filetype: Some(Filetype::PlainText),
            ..Default::default()
        }
    }

    #[test]
    fn missing_html_selector_fails_gracefully() {
        let config = Config {
            input: InputConfig {
                files: vec![
                    generate_invalid_file_missing_selector(),
                    generate_valid_file(),
                ],
                ..Default::default()
            },
            ..Default::default()
        };

        let build_results = build(&config).unwrap();

        assert_eq!(build_results.1.len(), 1);

        let error_msg = build_results.1.first().unwrap().to_string();

        assert!(
            error_msg.contains("HTML selector `.article` is not present in the file"),
            "{}",
            error_msg
        );
    }

    #[test]
    fn empty_contents_fails_gracefully() {
        let config = Config {
            input: InputConfig {
                files: vec![
                    generate_invalid_file_empty_contents(),
                    generate_valid_file(),
                ],
                ..Default::default()
            },
            ..Default::default()
        };

        let build_results = build(&config).unwrap();
        assert_eq!(build_results.1.len(), 1);

        let error_msg = build_results.1.first().unwrap().to_string();
        assert!(error_msg.contains("No words in word list"));
    }

    #[test]
    fn test_all_invalid_files_return_error() {
        let config = Config {
            input: InputConfig {
                files: vec![
                    generate_invalid_file_empty_contents(),
                    generate_invalid_file_missing_selector(),
                ],
                ..Default::default()
            },
            ..Default::default()
        };

        let build_error = build(&config).unwrap_err();

        assert_eq!(build_error, IndexGenerationError::NoValidFiles);
    }

    #[test]
    fn test_failing_file_does_not_halt_indexing() {
        let config = Config {
            input: InputConfig {
                files: vec![
                    generate_invalid_file_missing_selector(),
                    generate_valid_file(),
                ],
                ..Default::default()
            },
            ..Default::default()
        };

        assert_eq!(build(&config).unwrap().1.len(), 1);
        assert_eq!(build(&config).unwrap().0.entries.len(), 1);
    }
}
