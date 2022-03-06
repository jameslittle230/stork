use std::collections::HashMap;

mod fill_containers;
mod fill_intermediate_entries;
mod fill_stems;

mod annotated_words_from_string;
pub mod errors;
pub mod intermediate_entry;

use fill_containers::fill_containers;
use fill_intermediate_entries::fill_intermediate_entries;
use fill_stems::fill_stems;

use errors::{DocumentError, IndexGenerationError, WordListGenerationError};

use intermediate_entry::NormalizedEntry;

pub mod nudger;
use nudger::Nudger;

use crate::{config::Config, V3Index as Index};

use super::{Container, Entry, PassthroughConfig};

#[derive(Debug)]
pub struct BuildResult {
    pub index: Index,
    pub errors: Vec<DocumentError>,
}

pub fn build(config: &Config) -> Result<BuildResult, IndexGenerationError> {
    Nudger::from(config).print();

    let mut intermediate_entries: Vec<NormalizedEntry> = Vec::new();
    let mut document_errors: Vec<DocumentError> = Vec::new();
    fill_intermediate_entries(config, &mut intermediate_entries, &mut document_errors)?;

    if intermediate_entries.is_empty() {
        return Err(IndexGenerationError::NoValidFiles);
    }

    let mut stems: HashMap<String, Vec<String>> = HashMap::new();
    fill_stems(&intermediate_entries, &mut stems);

    let mut containers: HashMap<String, Container> = HashMap::new();
    fill_containers(config, &intermediate_entries, &stems, &mut containers);

    let entries: Vec<Entry> = intermediate_entries.iter().map(Entry::from).collect();

    let passthrough_config = PassthroughConfig {
        url_prefix: config.input.url_prefix.clone(),
        title_boost: config.input.title_boost.clone(),
        excerpt_buffer: config.output.excerpt_buffer,
        excerpts_per_result: config.output.excerpts_per_result,
        displayed_results_count: config.output.displayed_results_count,
    };

    let index = Index {
        entries,
        containers,
        config: passthrough_config,
    };

    Ok(BuildResult {
        index,
        errors: document_errors,
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

    use crate::config::*;

    use super::*;

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

        assert_eq!(build_results.errors.len(), 1);

        let expected = &WordListGenerationError::SelectorNotPresent(".article".to_string());
        let computed = &build_results
            .errors
            .first()
            .unwrap()
            .word_list_generation_error;

        assert_eq!(expected, computed);
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
        assert_eq!(build_results.errors.len(), 1);

        let expected = &WordListGenerationError::EmptyWordList;
        let computed = &build_results
            .errors
            .first()
            .unwrap()
            .word_list_generation_error;
        assert_eq!(expected, computed);
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

        assert_eq!(build(&config).unwrap().errors.len(), 1);
        assert_eq!(build(&config).unwrap().index.entries.len(), 1);
    }

    #[test]
    fn long_normalized_word_can_be_indexed() {
        // Bug reported in issue 227.
        // @TODO: Should the prefix aliaser handle long words differently? I'm not sure if
        // a search for `prism` or `csharp` will return any results with this input.
        let config = Config {
            input: InputConfig {
                files: vec![
                    File {
                        filetype: Some(Filetype::Markdown),
                        explicit_source: Some(DataSource::Contents(
                            "https://prismjs.com/download.html#themes=prism&languages=markup+css+clike+javascript+bash+c+csharp+cpp+go+java+markdown+python+scss+sql+toml+yaml&plugins=toolbar+copy-to-clipboard".to_string())),
                        ..Default::default()
                    }
                ],
                ..Default::default()
            },
            ..Default::default()
        };

        let build_results = build(&config).unwrap();
        assert!(build_results.errors.is_empty());
    }

    #[test]
    fn longer_normalized_word_can_be_indexed() {
        // Bug reported in issue 230.
        // @TODO: Should the prefix aliaser handle long words differently? I'm not sure if
        // a search for `prism` or `csharp` will return any results with this input.
        let config = Config {
            input: InputConfig {
                files: vec![
                    File {
                        filetype: Some(Filetype::Markdown),
                        explicit_source: Some(DataSource::Contents(
                            "https://upload.wikimedia.org/wikipedia/commons/thumb/b/b1/Official_Presidential_portrait_of_Thomas_Jefferson_%28by_Rembrandt_Peale%2C_1800%29%28cropped%29.jpg/390px-Official_Presidential_portrait_of_Thomas_Jefferson_%28by_Rembrandt_Peale%2C_1800%29%28cropped%29.jpg".to_string())),
                        ..Default::default()
                    }
                ],
                ..Default::default()
            },
            ..Default::default()
        };

        let build_results = build(&config).unwrap();
        assert!(build_results.errors.is_empty());
    }
}
