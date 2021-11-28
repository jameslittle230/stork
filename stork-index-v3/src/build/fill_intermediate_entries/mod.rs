use super::{DocumentError, WordListGenerationError};

mod data_source_readers;
use data_source_readers::read_from_data_source;

mod word_list_generators;
use word_list_generators::create_word_list;

mod frontmatter;
use self::frontmatter::parse_frontmatter;

use super::{IndexGenerationError, NormalizedEntry};
use indicatif::{ProgressBar, ProgressDrawTarget, ProgressIterator, ProgressStyle};
use std::{collections::HashMap, convert::TryInto};
use stork_config::{Config, DataSource, File, Filetype, InputConfig, OutputConfig, StemmingConfig};

use unicode_segmentation::UnicodeSegmentation;

/**
 * A `DataSourceReader` will output one of these once it's read from the data source.
 */
pub struct ReadResult {
    pub(super) buffer: String,

    /// If the filetype can be read from the data source, the value will be
    /// stored here. When the builder gets to a word list generator, it should
    /// use the filetype here if it's available.
    pub(super) filetype: Option<Filetype>,

    #[allow(dead_code)]
    pub(super) frontmatter_fields: Option<HashMap<String, String>>,
}

impl ReadResult {
    fn extract_frontmatter(&self, config: &ReaderConfig) -> Self {
        let handling = config
            .file
            .frontmatter_handling_override
            .as_ref()
            .unwrap_or(&config.global.frontmatter_handling);

        let (frontmatter_fields, buffer) = parse_frontmatter(handling, &self.buffer);

        ReadResult {
            buffer: buffer.to_string(),
            filetype: self.filetype.clone(),
            frontmatter_fields: Some(frontmatter_fields),
        }
    }
}

pub struct ReaderConfig {
    pub global: InputConfig,
    pub file: File,
    pub output: OutputConfig,
}

impl ReaderConfig {
    fn get_stem_algorithm(&self) -> Option<rust_stemmers::Algorithm> {
        let current_stem_config = self
            .file
            .stemming_override
            .as_ref()
            .unwrap_or(&self.global.stemming);

        match current_stem_config {
            StemmingConfig::Language(alg) => Some(*alg),
            StemmingConfig::None => None,
        }
    }
}

pub(super) fn fill_intermediate_entries(
    config: &Config,
    intermediate_entries: &mut Vec<NormalizedEntry>,
    document_errors: &mut Vec<DocumentError>,
) -> Result<(), IndexGenerationError> {
    if config.input.files.is_empty() {
        return Err(IndexGenerationError::NoFilesSpecified);
    }

    let progress_bar = build_progress_bar(config);

    for stork_file in config
        .input
        .files
        .iter()
        .progress_with(progress_bar.clone())
    {
        let reader_config = ReaderConfig {
            global: config.input.clone(),
            file: stork_file.clone(),
            output: config.output.clone(),
        };

        tick_progress_bar_with_filename(&progress_bar, &stork_file.title);

        let intermediate_entry_result: Result<NormalizedEntry, WordListGenerationError> =
            || -> Result<NormalizedEntry, WordListGenerationError> {
                let read_result = read_from_data_source(&reader_config)?;
                let annotated_word_list = create_word_list(&reader_config, &read_result)?;

                if annotated_word_list.word_list.is_empty() {
                    return Err(WordListGenerationError::EmptyWordList);
                }

                Ok(NormalizedEntry {
                    annotated_word_list,
                    stem_algorithm: reader_config.get_stem_algorithm(),
                    title: stork_file.title.clone(),
                    url: stork_file.url.clone(),
                    fields: reader_config.file.fields,
                })
            }();

        match intermediate_entry_result {
            Ok(ie) => {
                intermediate_entries.push(ie);
            }

            Err(e) => {
                let document_error = DocumentError {
                    file: stork_file.clone(),
                    word_list_generation_error: e,
                };

                if config.input.break_on_file_error {
                    return Err(IndexGenerationError::DocumentErrors(vec![document_error]));
                }

                document_errors.push(document_error)
            }
        };
    }

    if config.input.break_on_file_error && !document_errors.is_empty() {
        return Err(IndexGenerationError::DocumentErrors(
            document_errors.clone(),
        ));
    }

    Ok(())
}

fn build_progress_bar(config: &Config) -> ProgressBar {
    let progress_bar = ProgressBar::new((config.input.files.len()).try_into().unwrap()).with_style(
        ProgressStyle::default_bar()
            .template("[{elapsed}] {bar:40.cyan/blue} {pos}/{len} | {msg}")
            .progress_chars("##-"),
    );

    let url_file_count: u32 = config.input.files.iter().fold(0, |acc, file| {
        if let DataSource::URL(_) = file.source() {
            acc + 1
        } else {
            acc
        }
    });

    let progress_bar_draw_target = {
        match url_file_count {
            0 => ProgressDrawTarget::hidden(),
            _ => ProgressDrawTarget::stderr_nohz(),
        }
    };

    progress_bar.set_draw_target(progress_bar_draw_target);

    progress_bar
}

fn tick_progress_bar_with_filename(progress_bar: &ProgressBar, filename: &str) {
    let message = truncate_with_ellipsis_to_length(filename, 21, None);
    progress_bar.set_message(message);
    progress_bar.tick();
}

fn truncate_with_ellipsis_to_length(
    string: &str,
    length: usize,
    ellipsis_override: Option<&str>,
) -> String {
    let ellipsis = ellipsis_override.unwrap_or("...");

    let grapheme_iter = UnicodeSegmentation::graphemes(string, true);
    let short_message: String = grapheme_iter.clone().take(length).collect();
    let long_message: String = grapheme_iter.clone().take(length + 1).collect();

    let truncated = {
        let ellipsis = if short_message == long_message {
            ""
        } else {
            ellipsis
        };

        format!("{}{}", short_message, ellipsis)
    };

    truncated
}

#[cfg(test)]
mod tests {
    use super::{fill_intermediate_entries, truncate_with_ellipsis_to_length};
    use crate::build::errors::{DocumentError, IndexGenerationError, WordListGenerationError};
    use crate::build::intermediate_entry::NormalizedEntry;
    use stork_config::{Config, DataSource, File, InputConfig, OutputConfig};
    use unicode_segmentation::UnicodeSegmentation;

    #[test]
    fn break_on_file_error_breaks() {
        let mut invalid_file = File::default();
        invalid_file.explicit_source = Some(DataSource::Contents("".to_string())); // Empty word list error
        let mut input = InputConfig::default();
        input.files = vec![invalid_file];
        input.break_on_file_error = true;
        let output = OutputConfig::default();
        let config = Config { input, output };

        let mut intermediate_entries: Vec<NormalizedEntry> = vec![];
        let mut document_errors: Vec<DocumentError> = vec![];

        let r = fill_intermediate_entries(&config, &mut intermediate_entries, &mut document_errors)
            .err()
            .unwrap();
        if let IndexGenerationError::DocumentErrors(vec) = r {
            let word_list_generation_error = &vec[0].word_list_generation_error;
            assert_eq!(
                word_list_generation_error,
                &WordListGenerationError::EmptyWordList
            )
        } else {
            panic!("Result is {:?}", r);
        }
    }

    #[test]
    fn false_break_on_file_error_does_not_break() {
        let mut invalid_file = File::default();
        invalid_file.explicit_source = Some(DataSource::Contents("".to_string())); // Empty word list error
        let mut input = InputConfig::default();
        input.files = vec![invalid_file];
        input.break_on_file_error = false;
        let output = OutputConfig::default();
        let config = Config { input, output };

        let mut intermediate_entries: Vec<NormalizedEntry> = vec![];
        let mut document_errors: Vec<DocumentError> = vec![];

        let result =
            fill_intermediate_entries(&config, &mut intermediate_entries, &mut document_errors);
        assert!(result.is_ok());
        assert_eq!(document_errors.len(), 1);
        assert_eq!(
            document_errors[0].word_list_generation_error,
            WordListGenerationError::EmptyWordList
        );
    }

    #[test]
    fn test_truncate_with_ellipsis_on_naughty_strings() {
        // https://github.com/minimaxir/big-list-of-naughty-strings/blob/master/blns.txt#L152
        let naughty_strings = vec![
            "\u{c5}\u{cd}\u{ce}\u{cf}\u{2dd}\u{d3}\u{d4}\u{f8ff}\u{d2}\u{da}\u{c6}\u{2603}",
            "\u{152}\u{201e}\u{b4}\u{2030}\u{2c7}\u{c1}\u{a8}\u{2c6}\u{d8}\u{220f}\u{201d}\u{2019}",
            "`\u{2044}\u{20ac}\u{2039}\u{203a}\u{fb01}\u{fb02}\u{2021}\u{b0}\u{b7}\u{201a}\u{2014}\u{b1}",
            "\u{7530}\u{4e2d}\u{3055}\u{3093}\u{306b}\u{3042}\u{3052}\u{3066}\u{4e0b}\u{3055}\u{3044}",
            "\u{548c}\u{88fd}\u{6f22}\u{8a9e}",
            "\u{1f468}\u{200d}\u{1f469}\u{200d}\u{1f466} \u{1f468}\u{200d}\u{1f469}\u{200d}\u{1f467}\u{200d}\u{1f466} \u{1f468}\u{200d}\u{1f468}\u{200d}\u{1f466} \u{1f469}\u{200d}\u{1f469}\u{200d}\u{1f467} \u{1f468}\u{200d}\u{1f466} \u{1f468}\u{200d}\u{1f467}\u{200d}\u{1f466} \u{1f469}\u{200d}\u{1f466} \u{1f469}\u{200d}\u{1f467}\u{200d}\u{1f466}",
        ];

        for string in naughty_strings {
            let grapheme_count = UnicodeSegmentation::graphemes(string, true).count();

            for i in 0..(grapheme_count + 3) {
                let _truncation = truncate_with_ellipsis_to_length(string, i, None);
            }
        }
    }
}
