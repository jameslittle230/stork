// use super::{DocumentError, WordListGenerationError};

// mod data_source_readers;
// use data_source_readers::read_from_data_source;

// mod word_list_generators;
// use word_list_generators::create_word_list;

// mod frontmatter;
// use self::frontmatter::parse_frontmatter;

// use super::{BuildError, WordSegmentedDocument};
// use crate::config::{Config, File, Filetype, InputConfig, OutputConfig, StemmingConfig};
// use std::collections::HashMap;

// /**
//  * A `DataSourceReader` will output one of these once it's read from the data source.
//  */
// #[derive(Debug, PartialEq)]
// pub struct ReadResult {
//     pub(super) buffer: String,

//     /// If the filetype can be read from the data source, the value will be
//     /// stored here. When the builder gets to a word list generator, it should
//     /// use the filetype here if it's available.
//     pub(super) filetype: Option<Filetype>,

//     #[allow(dead_code)]
//     pub(super) frontmatter_fields: Option<HashMap<String, String>>,
// }

// impl ReadResult {
//     fn extract_frontmatter(&self, config: &ReaderConfig) -> Self {
//         let handling = config
//             .file
//             .frontmatter_handling_override
//             .as_ref()
//             .unwrap_or(&config.global.frontmatter_handling);

//         let (frontmatter_fields, buffer) = parse_frontmatter(handling, &self.buffer);

//         ReadResult {
//             buffer,
//             filetype: self.filetype.clone(),
//             frontmatter_fields: Some(frontmatter_fields),
//         }
//     }
// }

// pub struct ReaderConfig {
//     pub global: InputConfig,
//     pub file: File,
//     pub output: OutputConfig,
// }

// impl ReaderConfig {
//     fn get_stem_algorithm(&self) -> Option<rust_stemmers::Algorithm> {
//         let current_stem_config = self
//             .file
//             .stemming_override
//             .as_ref()
//             .unwrap_or(&self.global.stemming);

//         match current_stem_config {
//             StemmingConfig::Language(alg) => Some(*alg),
//             StemmingConfig::None => None,
//         }
//     }
// }

// pub(super) fn fill_intermediate_entries(
//     config: &Config,
//     intermediate_entries: &mut Vec<WordSegmentedDocument>,
//     document_errors: &mut Vec<DocumentError>,
// ) -> Result<(), BuildError> {
//     if config.input.files.is_empty() {
//         return Err(BuildError::NoFilesSpecified);
//     }

//     for stork_file in &config.input.files {
//         let reader_config = ReaderConfig {
//             global: config.input.clone(),
//             file: stork_file.clone(),
//             output: config.output.clone(),
//         };

//         let intermediate_entry_result: Result<WordSegmentedDocument, WordListGenerationError> =
//             || -> Result<WordSegmentedDocument, WordListGenerationError> {
//                 let read_result = read_from_data_source(&reader_config)?;
//                 let annotated_word_list = create_word_list(&reader_config, &read_result)?;

//                 if annotated_word_list.word_list.is_empty() {
//                     return Err(WordListGenerationError::EmptyWordList);
//                 }

//                 Ok(WordSegmentedDocument {
//                     annotated_word_list,
//                     stem_algorithm: reader_config.get_stem_algorithm(),
//                     title: stork_file.title.clone(),
//                     url: stork_file.url.clone(),
//                     fields: reader_config.file.fields,
//                 })
//             }();

//         match intermediate_entry_result {
//             Ok(ie) => {
//                 intermediate_entries.push(ie);
//             }

//             Err(e) => {
//                 let document_error = DocumentError {
//                     file: stork_file.clone(),
//                     word_list_generation_error: e,
//                 };

//                 if config.input.break_on_file_error {
//                     return Err(BuildError::PartialDocumentErrors(vec![document_error]));
//                 }

//                 document_errors.push(document_error);
//             }
//         };
//     }

//     if config.input.break_on_file_error && !document_errors.is_empty() {
//         return Err(BuildError::PartialDocumentErrors(document_errors.clone()));
//     }

//     Ok(())
// }

// #[cfg(test)]
// mod tests {
//     use super::fill_intermediate_entries;
//     use crate::{
//         build::{errors::WordListGenerationError, word_segmented_document::WordSegmentedDocument},
//         config::{Config, DataSource, File, InputConfig, OutputConfig},
//         BuildError, DocumentError,
//     };

//     #[test]
//     fn break_on_file_error_breaks() {
//         let invalid_file = File {
//             explicit_source: Some(DataSource::Contents("".to_string())), // Empty word list error,
//             ..File::default()
//         };

//         let input = InputConfig {
//             files: vec![invalid_file],
//             break_on_file_error: true,
//             ..InputConfig::default()
//         };

//         let output = OutputConfig::default();
//         let config = Config { input, output };

//         let mut intermediate_entries: Vec<WordSegmentedDocument> = vec![];
//         let mut document_errors: Vec<DocumentError> = vec![];

//         let r = fill_intermediate_entries(&config, &mut intermediate_entries, &mut document_errors)
//             .err()
//             .unwrap();
//         if let BuildError::PartialDocumentErrors(vec) = r {
//             let word_list_generation_error = &vec[0].word_list_generation_error;
//             assert_eq!(
//                 word_list_generation_error,
//                 &WordListGenerationError::EmptyWordList
//             );
//         } else {
//             panic!("Result is {:?}", r);
//         }
//     }

//     #[test]
//     fn false_break_on_file_error_does_not_break() {
//         let invalid_file = File {
//             explicit_source: Some(DataSource::Contents("".to_string())), // Empty word list error
//             ..File::default()
//         };

//         let input = InputConfig {
//             files: vec![invalid_file],
//             break_on_file_error: false,
//             ..InputConfig::default()
//         };

//         let output = OutputConfig::default();
//         let config = Config { input, output };

//         let mut intermediate_entries: Vec<WordSegmentedDocument> = vec![];
//         let mut document_errors: Vec<DocumentError> = vec![];

//         let result =
//             fill_intermediate_entries(&config, &mut intermediate_entries, &mut document_errors);
//         assert!(result.is_ok());
//         assert_eq!(document_errors.len(), 1);
//         assert_eq!(
//             document_errors[0].word_list_generation_error,
//             WordListGenerationError::EmptyWordList
//         );
//     }
// }
