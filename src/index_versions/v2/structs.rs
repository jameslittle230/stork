use super::scores::_MATCHED_WORD_SCORE;
use crate::IndexFromFile;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::TryInto;

pub type EntryIndex = usize;
pub type AliasTarget = String;
pub type Score = u8;
type Fields = Option<HashMap<String, String>>;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(super) struct Entry {
    pub(super) contents: String,
    pub(super) title: String,
    pub(super) url: String,
    pub(super) fields: Fields,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(super) struct SearchResult {
    pub(super) excerpts: Vec<Excerpt>,
    pub(super) score: Score,
}

impl SearchResult {
    pub(super) fn _new() -> SearchResult {
        SearchResult {
            excerpts: vec![],
            score: _MATCHED_WORD_SCORE,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(super) struct Excerpt {
    pub(super) word_index: usize,
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
#[derive(Serialize, Deserialize, Clone, Debug)]
pub(super) struct Container {
    pub(super) results: HashMap<EntryIndex, SearchResult>,
    pub(super) aliases: HashMap<AliasTarget, Score>,
}

impl Container {
    pub fn _new() -> Container {
        Container {
            results: HashMap::new(),
            aliases: HashMap::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Index {
    pub(super) entries: Vec<Entry>,
    pub(super) queries: HashMap<String, Container>,
}

impl Index {
    pub fn from_file(file: &IndexFromFile) -> Index {
        let (version_size_bytes, rest) = file.split_at(std::mem::size_of::<u64>());
        let version_size = u64::from_be_bytes(version_size_bytes.try_into().unwrap());
        let (_version_bytes, rest) = rest.split_at(version_size.try_into().unwrap());

        let (entries_size_bytes, rest) = rest.split_at(std::mem::size_of::<u64>());
        let entries_size = u64::from_be_bytes(entries_size_bytes.try_into().unwrap());
        let (entries_bytes, rest) = rest.split_at(entries_size.try_into().unwrap());
        let entries = bincode::deserialize(entries_bytes).unwrap();

        let (queries_size_bytes, rest) = rest.split_at(std::mem::size_of::<u64>());
        let queries_size = u64::from_be_bytes(queries_size_bytes.try_into().unwrap());
        let (queries_bytes, _rest) = rest.split_at(queries_size.try_into().unwrap());
        let queries = bincode::deserialize(queries_bytes).unwrap();

        Index { entries, queries }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::{BufReader, Read};

    #[test]
    fn can_parse_0_5_3_index() {
        let file = fs::File::open("./src/test-indexes/federalist-min-0.5.3.st").unwrap();
        let mut buf_reader = BufReader::new(file);
        let mut index_bytes: Vec<u8> = Vec::new();
        let _bytes_read = buf_reader.read_to_end(&mut index_bytes);
        let index = Index::from_file(index_bytes.as_slice());
        assert_eq!(1, index.entries.len());
        assert_eq!(2477, index.queries.len());
    }

    #[test]
    fn can_parse_0_6_0_index() {
        let file = fs::File::open("./src/test-indexes/federalist-min-0.6.0.st").unwrap();
        let mut buf_reader = BufReader::new(file);
        let mut index_bytes: Vec<u8> = Vec::new();
        let _bytes_read = buf_reader.read_to_end(&mut index_bytes);
        let index = Index::from_file(index_bytes.as_slice());
        assert_eq!(1, index.entries.len());
        assert_eq!(2477, index.queries.len());
    }
}
