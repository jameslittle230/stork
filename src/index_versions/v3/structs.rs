use super::builder::IntermediateEntry;
use super::scores::*;
use crate::config::TitleBoost;
use crate::IndexFromFile;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};

// extern crate htmlescape;
// use htmlescape::encode_minimal;

pub type EntryIndex = usize;
pub type AliasTarget = String;
pub type Score = u8;
pub type Fields = HashMap<String, String>;

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

impl WordListSource {
    #[allow(clippy::trivially_copy_pass_by_ref)]
    fn is_default(&self) -> bool {
        self == &WordListSource::default()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(super) struct AnnotatedWord {
    pub(super) word: String,
    pub(super) fields: Fields,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(super) struct Contents {
    pub(super) word_list: Vec<AnnotatedWord>,
}

impl Contents {
    pub(super) fn get_full_text(&self) -> String {
        self.word_list
            .iter()
            .map(|aw| aw.word.clone())
            .collect::<Vec<String>>()
            .join(" ")
        // encode_minimal(out.as_str())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(super) struct Entry {
    pub(super) contents: String,
    pub(super) title: String,
    pub(super) url: String,
    pub(super) fields: Fields,
}

impl From<&IntermediateEntry> for Entry {
    fn from(ie: &IntermediateEntry) -> Self {
        Entry {
            contents: ie.contents.get_full_text(),
            title: ie.title.clone(),
            url: ie.url.clone(),
            fields: ie.fields.clone(),
        }
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

    #[serde(default, skip_serializing_if = "WordListSource::is_default")]
    pub(super) source: WordListSource,

    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub(super) fields: Fields,
}

/**
 * A Container holds:
 *
 * - a HashMap of EntryIndexes to SearchResults
 * - a HashMap of AliasTargets to scores
 *
 * Each valid query should return a single Container. It is possible to derive
 * all search results for a given query from a single container.
 */
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub(super) struct Container {
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub(super) results: HashMap<EntryIndex, SearchResult>,

    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub(super) aliases: HashMap<AliasTarget, Score>,
}

impl Container {
    pub fn new() -> Container {
        Container::default()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub(super) struct PassthroughConfig {
    pub(super) url_prefix: String,
    pub(super) title_boost: TitleBoost,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Index {
    pub(super) config: PassthroughConfig,
    pub(super) entries: Vec<Entry>,
    pub(super) containers: HashMap<String, Container>,
}

impl TryFrom<&IndexFromFile> for Index {
    type Error = serde_cbor::error::Error;
    fn try_from(file: &IndexFromFile) -> Result<Self, Self::Error> {
        let (version_size_bytes, rest) = file.split_at(std::mem::size_of::<u64>());
        let version_size = u64::from_be_bytes(version_size_bytes.try_into().unwrap());
        let (_version_bytes, rest) = rest.split_at(version_size as usize);

        let (index_size_bytes, rest) = rest.split_at(std::mem::size_of::<u64>());
        let index_size = u64::from_be_bytes(index_size_bytes.try_into().unwrap());
        let (index_bytes, _rest) = rest.split_at(index_size as usize);

        serde_cbor::de::from_slice(index_bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::{BufReader, Read};

    #[test]
    fn can_parse_1_0_0_index() {
        let file = fs::File::open("./test-assets/federalist-min-1.0.0.st").unwrap();
        let mut buf_reader = BufReader::new(file);
        let mut index_bytes: Vec<u8> = Vec::new();
        let _bytes_read = buf_reader.read_to_end(&mut index_bytes);
        let index = Index::try_from(index_bytes.as_slice()).unwrap();
        assert_eq!(1, index.entries.len());
        assert_eq!(2477, index.containers.len());
    }
}
