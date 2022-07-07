#![allow(clippy::module_name_repetitions)]

use std::collections::{BTreeMap, HashMap};

#[cfg(feature = "display")]
use std::fmt::Display;

mod fill_containers;
mod fill_intermediate_entries;
mod fill_stems;

mod annotated_words_from_string;
pub mod errors;
pub mod word_segmented_document;

mod contents_reader;
use contents_reader::read_contents;

mod document_parser;
use document_parser::extract_document_contents;

use bytes::Bytes;

pub use errors::{BuildError, BuildWarning, BuildWarning::DocumentReadError};

use crate::{
    config::Config,
    index_v4::{Document, Excerpt, MetadataEntry},
    index_v4::{IndexDiskRepresentation, QueryResult},
};

use self::word_segmented_document::WordSegmentedDocument;

/**
 * Eventually, the build method will return this!
 */
pub struct BuildOutput {
    pub index: Vec<Bytes>, // The build process might create multiple files, hence the vec
    pub statistics: IndexBuildStatistics, // For reporting to our build frontend
    pub warnings: Vec<BuildWarning>, // Non-fatal issues
}

#[derive(Debug)]
pub struct IndexBuildStatistics {
    pub entries_count: usize,
    pub tokens_count: usize,
    pub index_size_bytes: usize,
}

#[cfg(feature = "display")]
impl Display for IndexBuildStatistics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let document_count_string = self.entries_count.to_formatted_string(&Locale::en);
        let tokens_count_string = self.tokens_count.to_formatted_string(&Locale::en);
        let bytes_per_document_string =
            (self.index_size_bytes / self.entries_count).to_formatted_string(&Locale::en);
        let bytes_per_token_string =
            (self.index_size_bytes / self.tokens_count).to_formatted_string(&Locale::en);
        f.write_fmt(format!(
            r#"Index stats:
  - {entries_count_string} entries
  - {tokens_count_string} search terms
  - {bytes_per_document_string} bytes per entry
  - {bytes_per_token_string} bytes per search term"#,
        ))
    }
}

#[allow(unused_variables, clippy::unnecessary_wraps)]

pub fn build_index(
    config: &Config,
    report_progress: Box<dyn Fn(f32)>,
) -> Result<BuildOutput, BuildError> {
    let mut build_warnings: Vec<BuildWarning> = Vec::new();

    let mut index = IndexDiskRepresentation::default();

    let mut word_document_map: BTreeMap<String, Vec<usize>> = BTreeMap::new();

    for document in &config.input.files {
        let read_contents_result = read_contents(document, config);

        if let Err(read_error) = &read_contents_result {
            build_warnings.push(DocumentReadError(read_error.clone()));
        }

        // The read contents contains the contents of the file as read, including
        // markup that needs to be parsed. It _does not_ contain frontmatter,
        // which is assumed to be "out of band" with the that markup and is always
        // assumed to be the first thing in the document.
        let read_contents = read_contents_result.unwrap();

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
        let document_parse_result = extract_document_contents(config, document, &read_contents);

        if let Err(parse_error) = &document_parse_result {
            build_warnings.push(DocumentReadError(parse_error.clone()));
        }

        let word_segmented_document = document_parse_result.unwrap();

        let document = make_document(&word_segmented_document, read_contents.frontmatter);
        let document_index = index.documents.len();
        index.documents.push(document);

        // Temporary word → vec<excerpt index> map, which the tree will be built from
        for word in &word_segmented_document.annotated_words {
            let query_results_index = index.query_results.len();
            index
                .query_results
                .push(QueryResult::DocumentContentsExcerpt(Excerpt {
                    document_id: document_index,
                    contents_character_offset: word.annotation.character_offset,
                }));

            word_document_map
                .entry(word.word.clone())
                .and_modify(|vec| vec.push(query_results_index))
                .or_insert_with(|| vec![query_results_index]);

            //     let excerpt = Excerpt {
            //         document_id: document_index,
            //         contents_character_offset: word.character_offset,
            //     };

            //     index
            //         .query_results
            //         .push(QueryResult::DocumentContentsExcerpt(excerpt));

            //     let query_result_index = index.query_results.len() - 1;

            // index
            //     .query_tree
            //     .insert_annotated_word(word, query_result_index);
        }
    }

    // for each document:
    //   1. fetch contents and extract frontmatter to fields
    //   2. create Document (v4 index model) and append to list
    //   3. segment contents into words
    //   4. for each word:
    //      a. create QueryResult (v4 index model) and append to list
    //      b. add word to query tree
    //      c. something something stems
    //   5. For each word in title:
    //      a. create QueryResult (v4 index model) and append to list
    //      b. add word to query tree
    //      c. something something stems
    //   6. Something something metadata

    // Generate statistics from index
    // Serialize index, wrap in envelope

    // dbg!(&word_document_map);

    for word in (&word_document_map).keys() {
        for query_result_index in word_document_map.get(word).unwrap() {
            if word == "printing" {
                dbg!(query_result_index);
                dbg!(index.query_results.get(query_result_index.to_owned()));
            }
            index
                .query_tree
                .push_value_for_word(word, query_result_index.to_owned());
        }
    }

    Ok(BuildOutput {
        index: vec![index.to_bytes()],
        statistics: IndexBuildStatistics {
            entries_count: 0,
            tokens_count: 0,
            index_size_bytes: 0,
        },
        warnings: build_warnings,
    })
}

fn make_document(
    word_segmented_document: &WordSegmentedDocument,
    metadata: Option<HashMap<String, String>>,
) -> Document {
    Document {
        title: word_segmented_document.title.clone(),
        contents: word_segmented_document.contents.clone(),
        url: word_segmented_document.url.clone(),
        metadata: make_metadata_from_map(metadata),
    }
}

fn make_metadata_from_map(hashmap: Option<HashMap<String, String>>) -> Vec<MetadataEntry> {
    let mut metadata_entries = Vec::new();

    if let Some(hashmap) = hashmap {
        for (k, v) in &hashmap {
            metadata_entries.push(MetadataEntry {
                key: k.clone(),
                value: v.clone(),
            });
        }
    }

    metadata_entries
}

// pub fn build(config: &Config) -> Result<BuildResult, BuildError> {
//     fill_intermediate_entries(config, &mut intermediate_entries, &mut document_errors)?;

//     if intermediate_entries.is_empty() {
//         if document_errors.is_empty() {
//             return Err(BuildError::NoFilesSpecified);
//         }
//         return Err(BuildError::AllDocumentErrors(document_errors));
//     }

//     let mut stems: BTreeMap<String, Vec<String>> = BTreeMap::new();
//     fill_stems(&intermediate_entries, &mut stems);

//     let mut containers: BTreeMap<String, Container> = BTreeMap::new();
//     fill_containers(config, &intermediate_entries, &stems, &mut containers);

//     let entries: Vec<Entry> = intermediate_entries
//         .iter()
//         .map(Entry::from)
//         .map(|mut entry| {
//             if config.output.excerpts_per_result == 0 {
//                 entry.contents = "".to_string();
//             }

//             entry
//         })
//         .collect::<Vec<Entry>>();

//     let passthrough_config = PassthroughConfig {
//         url_prefix: config.input.url_prefix.clone(),
//         title_boost: config.input.title_boost.clone(),
//         excerpt_buffer: config.output.excerpt_buffer,
//         excerpts_per_result: config.output.excerpts_per_result,
//         displayed_results_count: config.output.displayed_results_count,
//     };

//     let index = Index {
//         entries,
//         containers,
//         config: passthrough_config,
//     };

//     Ok(BuildResult {
//         index,
//         errors: document_errors,
//     })
// }

// #[cfg(test)]
// mod tests {

//     use crate::config::{Config, DataSource, File, Filetype, InputConfig};

//     use super::*;

//     use pretty_assertions::assert_eq;

//     fn generate_invalid_file_missing_selector() -> File {
//         File {
//             explicit_source: Some(DataSource::Contents("".to_string())),
//             title: "Missing Selector".to_string(),
//             filetype: Some(Filetype::HTML),
//             html_selector_override: Some(".article".to_string()),
//             ..File::default()
//         }
//     }

//     fn generate_invalid_file_empty_contents() -> File {
//         File {
//             explicit_source: Some(DataSource::Contents("".to_string())),
//             title: "Empty Contents".to_string(),
//             filetype: Some(Filetype::PlainText),
//             ..File::default()
//         }
//     }

//     fn generate_valid_file() -> File {
//         File {
//             explicit_source: Some(DataSource::Contents("This is contents".to_string())),
//             title: "Successful File".to_string(),
//             filetype: Some(Filetype::PlainText),
//             ..File::default()
//         }
//     }

//     #[test]
//     fn missing_html_selector_fails_gracefully() {
//         let config = Config {
//             input: InputConfig {
//                 files: vec![
//                     generate_invalid_file_missing_selector(),
//                     generate_valid_file(),
//                 ],
//                 ..InputConfig::default()
//             },
//             ..Config::default()
//         };

//         let build_results = build(&config).unwrap();

//         assert_eq!(build_results.errors.len(), 1);

//         let expected = &WordListGenerationError::SelectorNotPresent(".article".to_string());
//         let computed = &build_results
//             .errors
//             .first()
//             .unwrap()
//             .word_list_generation_error;

//         assert_eq!(expected, computed);
//     }

//     #[test]
//     fn empty_contents_fails_gracefully() {
//         let config = Config {
//             input: InputConfig {
//                 files: vec![
//                     generate_invalid_file_empty_contents(),
//                     generate_valid_file(),
//                 ],
//                 ..InputConfig::default()
//             },
//             ..Config::default()
//         };

//         let build_results = build(&config).unwrap();
//         assert_eq!(build_results.errors.len(), 1);

//         let expected = &WordListGenerationError::EmptyWordList;
//         let computed = &build_results
//             .errors
//             .first()
//             .unwrap()
//             .word_list_generation_error;
//         assert_eq!(expected, computed);
//     }

//     #[test]
//     fn test_all_invalid_files_return_error() {
//         let config = Config {
//             input: InputConfig {
//                 files: vec![
//                     generate_invalid_file_empty_contents(),
//                     generate_invalid_file_missing_selector(),
//                 ],
//                 ..InputConfig::default()
//             },
//             ..Config::default()
//         };

//         let build_error = build(&config).unwrap_err();

//         // partial equality, this doesn't check that the inner vecs are equal :(
//         assert_eq!(build_error, BuildError::AllDocumentErrors(vec![]));

//         if let BuildError::AllDocumentErrors(document_errors) = build_error {
//             let word_list_generation_errors: Vec<WordListGenerationError> = document_errors
//                 .iter()
//                 .map(|d| d.word_list_generation_error.clone())
//                 .collect();
//             assert_eq!(
//                 word_list_generation_errors,
//                 vec![
//                     WordListGenerationError::EmptyWordList,
//                     WordListGenerationError::SelectorNotPresent(".article".to_string())
//                 ]
//             );
//         } else {
//             panic!()
//         }
//     }

//     #[test]
//     fn test_no_files_returns_error() {
//         let config = Config {
//             input: InputConfig {
//                 files: vec![],
//                 ..InputConfig::default()
//             },
//             ..Config::default()
//         };
//         let build_error = build(&config).unwrap_err();

//         assert_eq!(build_error, BuildError::NoFilesSpecified);
//     }

//     #[test]
//     fn test_failing_file_does_not_halt_indexing() {
//         let config = Config {
//             input: InputConfig {
//                 files: vec![
//                     generate_invalid_file_missing_selector(),
//                     generate_valid_file(),
//                 ],
//                 ..InputConfig::default()
//             },
//             ..Config::default()
//         };

//         assert_eq!(build(&config).unwrap().errors.len(), 1);
//         assert_eq!(build(&config).unwrap().index.entries.len(), 1);
//     }

//     #[test]
//     fn long_normalized_word_can_be_indexed() {
//         // Bug reported in issue 227.
//         // @TODO: Should the prefix aliaser handle long words differently? I'm not sure if
//         // a search for `prism` or `csharp` will return any results with this input.
//         let config = Config {
//             input: InputConfig {
//                 files: vec![
//                     File {
//                         filetype: Some(Filetype::Markdown),
//                         explicit_source: Some(DataSource::Contents(
//                             "https://prismjs.com/download.html#themes=prism&languages=markup+css+clike+javascript+bash+c+csharp+cpp+go+java+markdown+python+scss+sql+toml+yaml&plugins=toolbar+copy-to-clipboard".to_string())),
//                         ..File::default()
//                     }
//                 ],
//                 ..InputConfig::default()
//             },
//             ..Config::default()
//         };

//         let build_results = build(&config).unwrap();
//         assert!(build_results.errors.is_empty());
//     }

//     #[test]
//     fn longer_normalized_word_can_be_indexed() {
//         // Bug reported in issue 230.
//         // @TODO: Should the prefix aliaser handle long words differently? I'm not sure if
//         // a search for `prism` or `csharp` will return any results with this input.
//         let config = Config {
//             input: InputConfig {
//                 files: vec![
//                     File {
//                         filetype: Some(Filetype::Markdown),
//                         explicit_source: Some(DataSource::Contents(
//                             "https://upload.wikimedia.org/wikipedia/commons/thumb/b/b1/Official_Presidential_portrait_of_Thomas_Jefferson_%28by_Rembrandt_Peale%2C_1800%29%28cropped%29.jpg/390px-Official_Presidential_portrait_of_Thomas_Jefferson_%28by_Rembrandt_Peale%2C_1800%29%28cropped%29.jpg".to_string())),
//                         ..File::default()
//                     }
//                 ],
//                 ..InputConfig::default()
//             },
//             ..Config::default()
//         };

//         let build_results = build(&config).unwrap();
//         assert!(build_results.errors.is_empty());
//     }
// }
