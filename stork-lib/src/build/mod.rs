#![allow(clippy::module_name_repetitions)]
#![allow(unused_variables, clippy::unnecessary_wraps, clippy::too_many_lines)]

use std::collections::{BTreeMap, HashSet};

#[cfg(feature = "display")]
use std::fmt::Display;

pub(crate) mod parse_document;
pub(crate) mod read_contents;

use crate::{build_output, config, envelope, fields::Fields, index_v4};

use self::parse_document::DocumentParseValue;

use bytes::Bytes;
use rust_stemmers::Stemmer;

pub(crate) fn build_index(
    config: &config::Config,
    progress: Option<&dyn Fn(build_output::ProgressReport)>,
) -> Result<build_output::BuildSuccessValue, build_output::errors::InternalBuildError> {
    let mut warnings: Vec<build_output::BuildWarning> = Vec::new();

    let mut index = index_v4::IndexDiskRepresentation::default();

    let mut word_document_map: BTreeMap<String, HashSet<index_v4::QueryResultIndex>> =
        BTreeMap::new();

    let should_report_progress = should_report_progress(config);

    for (document_id, file_config) in config.input.files.iter().enumerate() {
        if let Some(progress_fn) = progress {
            if should_report_progress {
                progress_fn(build_output::ProgressReport {
                    total_document_count: config.input.files.len(),
                    state: build_output::ProgressState::StartedDocument {
                        index: document_id,
                        title: file_config.title.clone(),
                    },
                })
            }
        }

        let word_segmentation_result =
            read_contents::read_contents(config, document_id).and_then(|read_value| {
                parse_document::parse_document(config, document_id, &read_value)
            });

        // The read contents contains the contents of the file as read, including
        // markup that needs to be parsed. It _does not_ contain frontmatter,
        // which is assumed to be "out of band" with the that markup and is always
        // assumed to be the first thing in the document.
        //
        // We can't work with our contents until we've turned "read contents"
        // into "document contents" by parsing the markup, because "read contents"
        // still has that markup in it. So at this point, we parse markup to get
        // our final contents that get written to the index.
        //
        // HOWEVER, we also need to segment our document into words here because
        // once we're reading markup, we're also at the point where we want to
        // extract metadata that gets attached to each word.
        //
        // Therefore, after in the markup parse step, we need to have our document's
        // final contents, along with an annotated word list.

        let output_document: index_v4::Document;
        match &word_segmentation_result {
            Ok(document) => {
                output_document = make_output_document(document);
                index.documents.insert(document_id, output_document);
            }
            Err(e) => {
                warnings.push(e.into());
                continue;
            }
        }

        let document_parse_value = word_segmentation_result.unwrap();

        for title_word in &document_parse_value.annotated_title_words {
            let query_results_index: index_v4::QueryResultIndex = index.query_results.len();
            index
                .query_results
                .push(index_v4::QueryResult::TitleExcerpt(
                    index_v4::TitleExcerpt {
                        document_id,
                        title_character_offset: title_word.annotation.byte_offset,
                    },
                ));

            word_document_map
                .entry(title_word.word.clone())
                .and_modify(|set| {
                    set.insert(query_results_index);
                })
                .or_insert_with(|| HashSet::from([query_results_index]));

            // Step 3: Insert (Stem, QueryResult) into the temporary map
            match &file_config
                .stemming_override
                .clone()
                .unwrap_or_else(|| config.input.stemming.clone())
            {
                crate::config::StemmingConfig::None => {}
                crate::config::StemmingConfig::Language(alg) => {
                    let stem = Stemmer::create(*alg).stem(&title_word.word).to_string();
                    word_document_map
                        .entry(stem.clone())
                        .and_modify(|set| {
                            set.insert(query_results_index);
                        })
                        .or_insert_with(|| HashSet::from([query_results_index]));
                }
            }
        }

        // Build up a temporary word → vec<excerpt index> map,
        // which the tree will be built from
        for word in &document_parse_value.annotated_words {
            // Step 1: Build a QueryResult for the word
            let query_results_index: index_v4::QueryResultIndex = index.query_results.len();
            index
                .query_results
                .push(index_v4::QueryResult::DocumentContentsExcerpt(
                    index_v4::DocumentContentsExcerpt {
                        document_id,
                        contents_character_offset: word.annotation.byte_offset,
                        url_suffix: word.annotation.url_suffix.clone(),
                        // debug: Some(vec![file_config.title.clone(), word.word.clone()].join(" - ")),
                    },
                ));

            // Step 2: Insert (Word, QueryResult) into a temporary map
            word_document_map
                .entry(word.word.clone())
                .and_modify(|set| {
                    set.insert(query_results_index);
                })
                .or_insert_with(|| HashSet::from([query_results_index]));

            // Step 3: Insert (Stem, QueryResult) into the temporary map
            match &file_config
                .stemming_override
                .clone()
                .unwrap_or_else(|| config.input.stemming.clone())
            {
                crate::config::StemmingConfig::None => {}
                crate::config::StemmingConfig::Language(alg) => {
                    let stem = Stemmer::create(*alg).stem(&word.word).to_string();
                    word_document_map
                        .entry(stem.clone())
                        .and_modify(|set| {
                            set.insert(query_results_index);
                        })
                        .or_insert_with(|| HashSet::from([query_results_index]));
                }
            }

            // TODO: Step 4: Include fuzzy matches
        }
    }

    // Insert the temp map into our index's radix tree
    for word in (&word_document_map).keys() {
        // All the places the word appears
        let query_result_indexes = word_document_map.get(word).unwrap();
        for query_result_index in query_result_indexes {
            index
                .query_tree
                .push_value_for_string(word, query_result_index.to_owned());
        }

        // TODO: stems, prefixes, metadata
    }

    if should_report_progress {
        if let Some(report_progress) = progress {
            report_progress(build_output::ProgressReport {
                total_document_count: config.input.files.len(),
                state: build_output::ProgressState::Finished,
            });
        }
    }

    let primary_data = match config.local.debug_output {
        true => Bytes::from(serde_json::to_string_pretty(&index).unwrap()),
        false => {
            envelope::Envelope::wrap(envelope::Prefix::StorkV4, vec![index.to_bytes()]).to_bytes()
        }
    };

    Ok(build_output::BuildSuccessValue {
        primary_data,
        sidecar_data: vec![], // TODO: Shard indexes
        statistics: build_output::BuildStatistics {
            // TODO: Fill out statistics
            entries_count: 0,
            tokens_count: 0,
            index_size_bytes: 0,
        },
        warnings,
    })
}

fn should_report_progress(config: &config::Config) -> bool {
    config.input.files.len() > 1000
        || config
            .input
            .files
            .iter()
            .any(|file| matches!(file.source(), crate::config::DataSource::URL(_)))
}

fn make_output_document(document: &DocumentParseValue) -> index_v4::Document {
    fn make_metadata_from_map(hashmap: &Option<Fields>) -> Vec<index_v4::MetadataEntry> {
        let mut metadata_entries = Vec::new();

        if let Some(hashmap) = hashmap {
            for (k, v) in hashmap {
                metadata_entries.push(index_v4::MetadataEntry {
                    key: k.clone(),
                    value: v.clone(),
                });
            }
        }

        metadata_entries
    }

    index_v4::Document {
        title: document.title.clone(),
        contents: document.contents.clone(),
        url: document.url.clone(),
        metadata: make_metadata_from_map(&document.fields),
    }
}

#[cfg(test)]
mod tests {

    // use crate::config::{Config, DataSource, File, Filetype, InputConfig};

    // use super::{
    //     output::{
    //         document_problem::DocumentProblem, error::BuildError, success::Value,
    //         warning::BuildWarning,
    //     },
    //     *,
    // };

    // use pretty_assertions::assert_eq;

    // fn generate_invalid_file_missing_selector() -> File {
    //     File {
    //         explicit_source: Some(DataSource::Contents("".to_string())),
    //         title: "Missing Selector".to_string(),
    //         filetype: Some(Filetype::HTML),
    //         html_selector_override: Some(".article".to_string()),
    //         ..File::default()
    //     }
    // }

    // fn generate_invalid_file_empty_contents() -> File {
    //     File {
    //         explicit_source: Some(DataSource::Contents("".to_string())),
    //         title: "Empty Contents".to_string(),
    //         filetype: Some(Filetype::PlainText),
    //         ..File::default()
    //     }
    // }

    // fn generate_valid_file() -> File {
    //     File {
    //         explicit_source: Some(DataSource::Contents("This is contents".to_string())),
    //         title: "Successful File".to_string(),
    //         filetype: Some(Filetype::PlainText),
    //         ..File::default()
    //     }
    // }

    // fn build(config: &Config) -> Result<Value, BuildError> {
    //     build_index(config, None)
    // }

    // fn read_error_from_build_warning(build_warning: BuildWarning) -> Option<DocumentProblem> {
    //     if let BuildWarning::DocumentProblem(attrib) = build_warning {
    //         return Some(attrib.read_error);
    //     }

    //     None
    // }

    // #[test]
    // fn missing_html_selector_fails_gracefully() {
    //     let config = Config {
    //         input: InputConfig {
    //             files: vec![
    //                 generate_invalid_file_missing_selector(),
    //                 generate_valid_file(),
    //             ],
    //             ..InputConfig::default()
    //         },
    //         ..Config::default()
    //     };

    //     let build_results = build(&config).unwrap();

    //     assert_eq!(build_results.warnings.len(), 1);

    //     let expected = DocumentProblem::SelectorNotPresent(".article".to_string());
    //     let computed =
    //         read_error_from_build_warning(build_results.warnings.first().unwrap().to_owned())
    //             .unwrap();

    //     assert_eq!(expected, computed);
    // }

    // #[test]
    // fn empty_contents_fails_gracefully() {
    //     let config = Config {
    //         input: InputConfig {
    //             files: vec![
    //                 generate_invalid_file_empty_contents(),
    //                 generate_valid_file(),
    //             ],
    //             ..InputConfig::default()
    //         },
    //         ..Config::default()
    //     };

    //     let build_results = build(&config).unwrap();
    //     assert_eq!(build_results.errors.len(), 1);

    //     let expected = &WordListGenerationError::EmptyWordList;
    //     let computed = &build_results
    //         .errors
    //         .first()
    //         .unwrap()
    //         .word_list_generation_error;
    //     assert_eq!(expected, computed);
    // }

    // #[test]
    // fn test_all_invalid_files_return_error() {
    //     let config = Config {
    //         input: InputConfig {
    //             files: vec![
    //                 generate_invalid_file_empty_contents(),
    //                 generate_invalid_file_missing_selector(),
    //             ],
    //             ..InputConfig::default()
    //         },
    //         ..Config::default()
    //     };

    //     let build_error = build(&config).unwrap_err();

    //     // partial equality, this doesn't check that the inner vecs are equal :(
    //     assert_eq!(build_error, BuildError::AllDocumentErrors(vec![]));

    //     if let BuildError::AllDocumentErrors(document_errors) = build_error {
    //         let word_list_generation_errors: Vec<WordListGenerationError> = document_errors
    //             .iter()
    //             .map(|d| d.word_list_generation_error.clone())
    //             .collect();
    //         assert_eq!(
    //             word_list_generation_errors,
    //             vec![
    //                 WordListGenerationError::EmptyWordList,
    //                 WordListGenerationError::SelectorNotPresent(".article".to_string())
    //             ]
    //         );
    //     } else {
    //         panic!()
    //     }
    // }

    // #[test]
    // fn test_no_files_returns_error() {
    //     let config = Config {
    //         input: InputConfig {
    //             files: vec![],
    //             ..InputConfig::default()
    //         },
    //         ..Config::default()
    //     };
    //     let build_error = build(&config).unwrap_err();

    //     assert_eq!(build_error, BuildError::NoFilesSpecified);
    // }

    // #[test]
    // fn test_failing_file_does_not_halt_indexing() {
    //     let config = Config {
    //         input: InputConfig {
    //             files: vec![
    //                 generate_invalid_file_missing_selector(),
    //                 generate_valid_file(),
    //             ],
    //             ..InputConfig::default()
    //         },
    //         ..Config::default()
    //     };

    //     assert_eq!(build(&config).unwrap().errors.len(), 1);
    //     assert_eq!(build(&config).unwrap().index.entries.len(), 1);
    // }

    // #[test]
    // fn long_normalized_word_can_be_indexed() {
    //     // Bug reported in issue 227.
    //     // @TODO: Should the prefix aliaser handle long words differently? I'm not sure if
    //     // a search for `prism` or `csharp` will return any results with this input.
    //     let config = Config {
    //         input: InputConfig {
    //             files: vec![
    //                 File {
    //                     filetype: Some(Filetype::Markdown),
    //                     explicit_source: Some(DataSource::Contents(
    //                         "https://prismjs.com/download.html#themes=prism&languages=markup+css+clike+javascript+bash+c+csharp+cpp+go+java+markdown+python+scss+sql+toml+yaml&plugins=toolbar+copy-to-clipboard".to_string())),
    //                     ..File::default()
    //                 }
    //             ],
    //             ..InputConfig::default()
    //         },
    //         ..Config::default()
    //     };

    //     let build_results = build(&config).unwrap();
    //     assert!(build_results.errors.is_empty());
    // }

    // #[test]
    // fn longer_normalized_word_can_be_indexed() {
    //     // Bug reported in issue 230.
    //     // @TODO: Should the prefix aliaser handle long words differently? I'm not sure if
    //     // a search for `prism` or `csharp` will return any results with this input.
    //     let config = Config {
    //         input: InputConfig {
    //             files: vec![
    //                 File {
    //                     filetype: Some(Filetype::Markdown),
    //                     explicit_source: Some(DataSource::Contents(
    //                         "https://upload.wikimedia.org/wikipedia/commons/thumb/b/b1/Official_Presidential_portrait_of_Thomas_Jefferson_%28by_Rembrandt_Peale%2C_1800%29%28cropped%29.jpg/390px-Official_Presidential_portrait_of_Thomas_Jefferson_%28by_Rembrandt_Peale%2C_1800%29%28cropped%29.jpg".to_string())),
    //                     ..File::default()
    //                 }
    //             ],
    //             ..InputConfig::default()
    //         },
    //         ..Config::default()
    //     };

    //     let build_results = build(&config).unwrap();
    //     assert!(build_results.errors.is_empty());
    // }
}
