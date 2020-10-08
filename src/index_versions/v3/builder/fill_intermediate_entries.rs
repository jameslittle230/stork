use super::frontmatter::parse_frontmatter;
use super::word_list_generators::returns_word_list_generator;
use super::{Contents, IndexGenerationError, IntermediateEntry};
use crate::config::{Config, DataSource, StemmingConfig};
use rust_stemmers::Algorithm;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

pub fn fill_intermediate_entries(
    config: &Config,
    intermediate_entries: &mut Vec<IntermediateEntry>,
) -> Result<(), IndexGenerationError> {
    if config.input.files.is_empty() {
        return Err(IndexGenerationError::NoFilesSpecified);
    }

    let base_directory = Path::new(&config.input.base_directory);
    for stork_file in config.input.files.iter() {
        let filetype = &stork_file.computed_filetype().unwrap_or_else(|| panic!("Cannot determine a filetype for {}. Please include a filetype field in your config file or use a known file extension.", &stork_file.title));

        let buffer: String = match &stork_file.source {
            DataSource::Contents(contents) => contents.to_string(),
            DataSource::FilePath(path_string) => {
                let full_pathname = &base_directory.join(&path_string);
                let file = File::open(&full_pathname).map_err(|_| {
                    IndexGenerationError::FileNotFoundError(full_pathname.to_owned())
                })?;
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

        let mut per_file_input_config = config.input.clone();
        per_file_input_config.html_selector = stork_file.html_selector_override.clone();
        per_file_input_config.frontmatter_handling = stork_file
            .frontmatter_handling_override
            .clone()
            .unwrap_or(per_file_input_config.frontmatter_handling);

        let (frontmatter_fields, buffer) = parse_frontmatter(&per_file_input_config, &buffer);

        let contents: Contents = returns_word_list_generator(filetype)
            .create_word_list(&per_file_input_config, buffer.as_str())
            .map_err(IndexGenerationError::WordListGenerationError)?;

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

    Ok(())
}
