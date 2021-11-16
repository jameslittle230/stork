use super::scores::MATCHED_WORD_SCORE;
use crate::config::TitleBoost;
use crate::{
    common::{Fields, InternalWordAnnotation},
    config::OutputConfig,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type EntryIndex = usize;
pub type AliasTarget = String;
pub type Score = u8;

/**
 * A serialized Index, for all intents and purposes, is the whole contents of
 * a Stork index file.
 */
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Index {
    pub(super) config: PassthroughConfig,
    pub(super) entries: Vec<Entry>,
    pub(super) containers: HashMap<String, Container>,
}

impl Index {
    pub fn entries_len(&self) -> usize {
        self.entries.len()
    }

    pub fn avg_entry_size(&self) -> usize {
        self.entries
            .iter()
            .map(|entry| entry.contents.len())
            .sum::<usize>()
            / self.entries_len()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(super) struct PassthroughConfig {
    pub(super) url_prefix: String,
    pub(super) title_boost: TitleBoost,
    pub(super) excerpt_buffer: u8,
    pub(super) excerpts_per_result: u8,
    pub(super) displayed_results_count: u8,
}

impl Default for PassthroughConfig {
    fn default() -> Self {
        let output_config = OutputConfig::default();
        Self {
            url_prefix: String::default(),
            title_boost: TitleBoost::default(),
            excerpt_buffer: output_config.excerpt_buffer,
            excerpts_per_result: output_config.excerpts_per_result,
            displayed_results_count: output_config.displayed_results_count,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(super) struct Entry {
    pub(super) contents: String,
    pub(super) title: String,
    pub(super) url: String,
    pub(super) fields: Fields,
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
    pub(super) results: HashMap<EntryIndex, SearchResult>,

    // #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub(super) aliases: HashMap<AliasTarget, Score>,
}

impl Container {
    pub fn new() -> Container {
        Container::default()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(super) struct SearchResult {
    pub(super) excerpts: Vec<Excerpt>,
    pub(super) score: Score,
}

impl SearchResult {
    pub(super) fn new() -> SearchResult {
        SearchResult {
            excerpts: vec![],
            score: MATCHED_WORD_SCORE,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub(super) struct Excerpt {
    pub(super) word_index: usize,

    // #[serde(default, skip_serializing_if = "WordListSource::is_default")]
    pub(super) source: WordListSource,

    // #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(super) internal_annotations: Vec<InternalWordAnnotation>,

    // #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub(super) fields: Fields,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum WordListSource {
    Title,
    Contents,
}

impl Default for WordListSource {
    fn default() -> Self {
        WordListSource::Contents
    }
}

// impl WordListSource {
//     #[allow(clippy::trivially_copy_pass_by_ref)]
//     fn is_default(&self) -> bool {
//         self == &WordListSource::default()
//     }
// }

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub(super) struct AnnotatedWord {
    pub(super) word: String,
    pub(super) internal_annotations: Vec<InternalWordAnnotation>,
    pub(super) fields: Fields,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AnnotatedWordList {
    pub(super) word_list: Vec<AnnotatedWord>,
}

impl AnnotatedWordList {
    pub(super) fn get_full_text(&self) -> String {
        self.word_list
            .iter()
            .map(|aw| aw.word.clone())
            .collect::<Vec<String>>()
            .join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;
    use std::fs;
    use std::io::{BufReader, Read};

    #[test]
    fn can_parse_0_7_0_index() {
        let file = fs::File::open("./src/test-indexes/federalist-min-0.7.0.st").unwrap();
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
                    ..Default::default()
                },
                AnnotatedWord {
                    word: "is-a".to_string(),
                    internal_annotations: vec![InternalWordAnnotation::UrlSuffix("a".to_string())],
                    fields: HashMap::default(),
                },
                AnnotatedWord {
                    word: "set".to_string(),
                    ..Default::default()
                },
                AnnotatedWord {
                    word: "of".to_string(),
                    ..Default::default()
                },
                AnnotatedWord {
                    word: "words.".to_string(),
                    ..Default::default()
                },
            ],
        }
        .get_full_text();

        assert_eq!(intended, generated);
    }
}
