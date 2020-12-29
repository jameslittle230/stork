use super::{frontmatter::parse_frontmatter, word_list_generators::WordListGenerationError};
use super::{word_list_generators::returns_word_list_generator, DocumentError};
use super::{IndexGenerationError, IntermediateEntry};
use crate::config::{Config, DataSource, StemmingConfig};
use rust_stemmers::Algorithm;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

pub(super) fn fill_intermediate_entries(
    config: &Config,
    intermediate_entries: &mut Vec<IntermediateEntry>,
    document_errors: &mut Vec<DocumentError>,
) -> Result<(), IndexGenerationError> {
    if config.input.files.is_empty() {
        return Err(IndexGenerationError::NoFilesSpecified);
    }

    let base_directory = Path::new(&config.input.base_directory);

    let mut per_file_input_config = config.input.clone();

    for stork_file in config.input.files.iter() {
        let perhaps_filetype = &stork_file.computed_filetype();
        let filetype = match perhaps_filetype {
            Some(filetype) => filetype,
            None => {
                document_errors.push(DocumentError {
                    file: stork_file.clone(),
                    word_list_generation_error: WordListGenerationError::CannotDetermineFiletype,
                });
                continue;
            }
        };

        let buffer: String = match &stork_file.source {
            DataSource::Contents(contents) => contents.to_string(),
            DataSource::FilePath(path_string) => {
                let full_pathname = &base_directory.join(&path_string);
                let file = match File::open(&full_pathname) {
                    Ok(file) => file,
                    Err(_e) => {
                        document_errors.push(DocumentError {
                            file: stork_file.clone(),
                            word_list_generation_error: WordListGenerationError::FileNotFound,
                        });
                        continue;
                    }
                };
                let mut buf_reader = BufReader::new(file);
                let mut buffer = String::new();
                let _bytes_read = buf_reader.read_to_string(&mut buffer);
                buffer
            }
            DataSource::URL(_url) => panic!("URL not available yet"),
        };

        let current_stem_config = stork_file
            .stemming_override
            .clone()
            .unwrap_or_else(|| config.input.stemming.clone());

        let stem_algorithm: Option<Algorithm> = match current_stem_config {
            StemmingConfig::Language(alg) => Some(alg.to_owned()),
            StemmingConfig::None => None,
        };

        per_file_input_config.html_selector = stork_file.html_selector_override.clone();
        per_file_input_config.frontmatter_handling = stork_file
            .frontmatter_handling_override
            .clone()
            .unwrap_or(per_file_input_config.frontmatter_handling);

        let (frontmatter_fields, buffer) = parse_frontmatter(&per_file_input_config, &buffer);

        let word_list_result = returns_word_list_generator(filetype)
            .create_word_list(&per_file_input_config, buffer.as_str());

        match word_list_result {
            Ok(contents) => {
                let mut fields = stork_file.fields.clone();
                fields.extend(frontmatter_fields.into_iter());

                let entry = IntermediateEntry {
                    contents,
                    stem_algorithm,
                    title: stork_file.title.clone(),
                    url: stork_file.url.clone(),
                    fields,
                };

                intermediate_entries.push(entry);
            }
            Err(error) => document_errors.push(DocumentError {
                file: stork_file.clone(),
                word_list_generation_error: error,
            }),
        }
    }

    Ok(())
}
