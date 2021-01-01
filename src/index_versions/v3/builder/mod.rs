use super::structs::*;
use crate::config::Config;
use std::collections::HashMap;

mod fill_containers;
mod fill_intermediate_entries;
mod fill_stems;

mod annotated_words_from_string;
pub mod errors;

pub mod intermediate_entry;
mod word_list_generators;

use fill_containers::fill_containers;
use fill_intermediate_entries::fill_intermediate_entries;
use fill_stems::fill_stems;

use errors::{DocumentError, IndexGenerationError};

use intermediate_entry::IntermediateEntry;

pub mod nudger;
use nudger::Nudger;

pub mod frontmatter;

extern crate rust_stemmers;

pub fn build(config: &Config) -> Result<(Index, Vec<DocumentError>), IndexGenerationError> {
    println!("{}", Nudger::from(config).generate_formatted_output());

    let mut intermediate_entries: Vec<IntermediateEntry> = Vec::new();
    let mut document_errors: Vec<DocumentError> = Vec::new();
    fill_intermediate_entries(&config, &mut intermediate_entries, &mut document_errors)?;

    if !document_errors.is_empty() {
        println!(
            "{} error{} while indexing files:",
            document_errors.len(),
            match document_errors.len() {
                1 => "",
                _ => "s",
            }
        )
    }
    for error in &document_errors {
        println!("- {}", &error);
    }

    let mut stems: HashMap<String, Vec<String>> = HashMap::new();
    fill_stems(&intermediate_entries, &mut stems);

    let mut containers: HashMap<String, Container> = HashMap::new();
    fill_containers(&config, &intermediate_entries, &stems, &mut containers);

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
            entries,
            containers,
            config,
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
    use crate::config::*;

    #[test]
    fn test_not_present_html_selector_fails_gracefully() {
        let config = Config {
            input: InputConfig {
                files: vec![File {
                    source: DataSource::Contents("".to_string()),
                    title: "Title".to_string(),
                    filetype: Some(Filetype::HTML),
                    html_selector_override: Some(".article".to_string()),
                    ..Default::default()
                }],
                ..Default::default()
            },
            ..Default::default()
        };

        assert_eq!(build(&config).unwrap().1.len(), 1);

        assert_eq!(
            build(&config).unwrap().1.first().unwrap().to_string(),
            "Error indexing `Title`: HTML selector `.article` is not present in the file."
        );
    }

    #[test]
    fn test_failing_file_does_not_halt_indexing() {
        let config = Config {
            input: InputConfig {
                files: vec![
                    File {
                        source: DataSource::Contents("".to_string()),
                        title: "Title".to_string(),
                        filetype: Some(Filetype::HTML),
                        html_selector_override: Some(".article".to_string()),
                        ..Default::default()
                    },
                    File {
                        source: DataSource::Contents("".to_string()),
                        title: "Title 2".to_string(),
                        filetype: Some(Filetype::PlainText),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            ..Default::default()
        };

        assert_eq!(build(&config).unwrap().1.len(), 1);
        assert_eq!(build(&config).unwrap().0.entries.len(), 1);
    }
}
