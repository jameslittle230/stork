use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use smart_default::SmartDefault;

type EntryIndex = usize;
type AliasTarget = String;
type Score = u8;

mod read;
pub(crate) mod scores;
mod search;

pub use search::search;

use crate::config::{OutputConfig, TitleBoost};
use crate::{Fields, InternalWordAnnotation};

mod write;

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Index {
    pub(crate) config: PassthroughConfig,
    pub(crate) entries: Vec<Entry>,
    pub(crate) containers: BTreeMap<String, Container>,
}

impl Index {
    pub fn entries_len(&self) -> usize {
        self.entries.len()
    }

    pub fn word_count(&self) -> usize {
        self.entries.iter().map(|entry| entry.contents.len()).sum()
    }

    pub fn avg_entry_size(&self) -> usize {
        self.word_count() / self.entries_len()
    }

    pub fn search_term_count(&self) -> usize {
        self.containers.keys().count()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, SmartDefault)]
pub(crate) struct PassthroughConfig {
    pub(crate) url_prefix: String,
    pub(crate) title_boost: TitleBoost,

    #[default(OutputConfig::default().excerpt_buffer)]
    pub(crate) excerpt_buffer: u8,
    #[default(OutputConfig::default().excerpts_per_result)]
    pub(crate) excerpts_per_result: u8,
    #[default(OutputConfig::default().displayed_results_count)]
    pub(crate) displayed_results_count: u8,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub(crate) struct Entry {
    pub(crate) contents: String,
    pub(crate) title: String,
    pub(crate) url: String,
    pub(crate) fields: Fields,
}

/**
 * A Container holds:
 *
 * - a `HashMap` of `EntryIndexes` to `SearchResults`
 * - a `HashMap` of `AliasTargets` to scores
 *
 * Each valid query should return a single Container. It is possible to derive
 * all search results for a given query from a single container.
 */
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Container {
    // #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub(crate) results: BTreeMap<EntryIndex, SearchResult>,

    // #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub(crate) aliases: BTreeMap<AliasTarget, Score>,
}

impl Container {
    #[cfg(feature = "build")]
    pub(crate) fn new() -> Self {
        Self::default()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct SearchResult {
    pub(crate) excerpts: Vec<Excerpt>,
    pub(crate) score: Score,
}

impl SearchResult {
    #[cfg(feature = "build")]
    pub(crate) fn new() -> SearchResult {
        SearchResult {
            excerpts: vec![],
            score: scores::MATCHED_WORD_SCORE,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub(crate) struct Excerpt {
    pub(crate) word_index: usize,

    // #[serde(default, skip_serializing_if = "WordListSource::is_default")]
    pub(crate) source: WordListSource,

    // #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) internal_annotations: Vec<InternalWordAnnotation>,

    // #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub(crate) fields: Fields,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, SmartDefault)]
pub(crate) enum WordListSource {
    Title,

    #[default]
    Contents,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub(crate) struct AnnotatedWord {
    pub(crate) word: String,
    pub(crate) internal_annotations: Vec<InternalWordAnnotation>,
    pub(crate) fields: Fields,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct AnnotatedWordList {
    pub(crate) word_list: Vec<AnnotatedWord>,
}

#[cfg(feature = "build")]
impl AnnotatedWordList {
    pub(crate) fn get_full_text(&self) -> String {
        self.word_list
            .iter()
            .map(|aw| aw.word.clone())
            .collect::<Vec<String>>()
            .join(" ")
    }
}

#[cfg(test)]
mod tests {
    use crate::config::{DataSource, File, Filetype, InputConfig};
    use crate::Config;

    use super::*;
    use pretty_assertions::assert_eq;

    use std::collections::HashMap;
    use std::convert::TryFrom;
    use std::fs;
    use std::io::{BufReader, Read};

    #[test]
    fn can_parse_0_7_0_index() {
        let file = fs::File::open("../test-assets/federalist-min-0.7.0.st").unwrap();
        let mut buf_reader = BufReader::new(file);
        let mut index_bytes: Vec<u8> = Vec::new();
        let _bytes_read = buf_reader.read_to_end(&mut index_bytes);
        let index = Index::try_from(index_bytes.as_slice()).unwrap();
        assert_eq!(1, index.entries.len());
        assert_eq!(2477, index.containers.len());
    }

    #[test]
    fn get_full_text() {
        let intended = "This is-a set of words.".to_string();
        let generated = AnnotatedWordList {
            word_list: vec![
                AnnotatedWord {
                    word: "This".to_string(),
                    ..AnnotatedWord::default()
                },
                AnnotatedWord {
                    word: "is-a".to_string(),
                    internal_annotations: vec![InternalWordAnnotation::UrlSuffix("a".to_string())],
                    fields: HashMap::default(),
                },
                AnnotatedWord {
                    word: "set".to_string(),
                    ..AnnotatedWord::default()
                },
                AnnotatedWord {
                    word: "of".to_string(),
                    ..AnnotatedWord::default()
                },
                AnnotatedWord {
                    word: "words.".to_string(),
                    ..AnnotatedWord::default()
                },
            ],
        }
        .get_full_text();

        assert_eq!(intended, generated);
    }

    #[test]
    fn index_with_zero_excerpts_per_result_is_smaller() {
        let config = Config {
            input: crate::config::InputConfig {
                files: vec![
                    File {
                        explicit_source: Some(DataSource::Contents(
                            "The quick brown fox jumps over the lazy dog.".to_string(),
                        )),
                        title: "Quick Brown Fox".to_string(),
                        filetype: Some(Filetype::PlainText),
                        ..File::default()
                    },
                    File {
                        explicit_source: Some(DataSource::Contents(
                            "Sphinx of black quartz, judge my vow".to_string(),
                        )),
                        title: "Sphinx of Black Quartz".to_string(),
                        filetype: Some(Filetype::PlainText),
                        ..File::default()
                    },
                ],
                ..InputConfig::default()
            },
            output: OutputConfig {
                excerpts_per_result: 0,
                ..OutputConfig::default()
            },
        };

        let build_result = crate::build(&config).unwrap();

        assert_eq!(build_result.index.containers.keys().len(), 33);

        assert!(build_result.index.containers.values().all(|container| {
            container
                .results
                .values()
                .all(|search_result| search_result.excerpts.is_empty())
        }));

        assert!(build_result
            .index
            .entries
            .into_iter()
            .all(|entry| entry.contents.is_empty()));
    }
}
