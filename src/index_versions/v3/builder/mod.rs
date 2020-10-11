use super::structs::*;
use crate::config::Config;
use std::fmt;
use std::{collections::HashMap, error::Error, path::PathBuf};

mod fill_intermediate_entries;
use fill_intermediate_entries::fill_intermediate_entries;

mod fill_stems;
use fill_stems::fill_stems;

mod fill_containers;
use fill_containers::fill_containers;

pub mod word_list_generators;
use word_list_generators::WordListGenerationError;

pub mod intermediate_entry;
use intermediate_entry::IntermediateEntry;

pub mod nudger;
use nudger::Nudger;

pub mod frontmatter;

extern crate rust_stemmers;

#[derive(Debug)]
pub enum IndexGenerationError {
    NoFilesSpecified,
    FileNotFoundError(PathBuf),
    WordListGenerationError(WordListGenerationError),
}

impl Error for IndexGenerationError {}

impl fmt::Display for IndexGenerationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let desc: String = match self {
            IndexGenerationError::NoFilesSpecified => {
                "No files specified in config file".to_string()
            }
            IndexGenerationError::FileNotFoundError(s) => {
                format!("File {} not found", s.to_string_lossy())
            }
            IndexGenerationError::WordListGenerationError(e) => e.to_string(),
        };

        write!(f, "{}", desc)
    }
}

pub fn build(config: &Config) -> Result<Index, IndexGenerationError> {
    println!("{}", Nudger::from(config).generate_formatted_output());

    let mut intermediate_entries: Vec<IntermediateEntry> = Vec::new();
    fill_intermediate_entries(&config, &mut intermediate_entries)?;

    let mut stems: HashMap<String, Vec<String>> = HashMap::new();
    fill_stems(&intermediate_entries, &mut stems);

    let mut containers: HashMap<String, Container> = HashMap::new();
    fill_containers(&intermediate_entries, &stems, &mut containers);

    let entries: Vec<Entry> = intermediate_entries.iter().map(Entry::from).collect();

    let config = PassthroughConfig {
        url_prefix: config.input.url_prefix.clone(),
        title_boost: config.input.title_boost.clone(),
        excerpt_buffer: config.output.excerpt_buffer,
        excerpts_per_result: config.output.excerpts_per_result,
        displayed_results_count: config.output.displayed_results_count,
    };

    Ok(Index {
        entries,
        containers,
        config,
    })
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

        build(&config).expect_err("Config didn't error when it should have!");
    }
}
