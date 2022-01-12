mod scores;
mod search;

use bytes::{Buf, Bytes};
use scores::_MATCHED_WORD_SCORE;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use stork_shared::StorkIndex;

type EntryIndex = usize;
type AliasTarget = String;
type Score = u8;
type Fields = Option<HashMap<String, String>>;

pub use search::search;

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Entry {
    contents: String,
    title: String,
    url: String,
    fields: Fields,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct SearchResult {
    excerpts: Vec<Excerpt>,
    score: Score,
}

impl SearchResult {
    fn _new() -> SearchResult {
        SearchResult {
            excerpts: vec![],
            score: _MATCHED_WORD_SCORE,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Excerpt {
    word_index: usize,
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
struct Container {
    results: HashMap<EntryIndex, SearchResult>,
    aliases: HashMap<AliasTarget, Score>,
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
    entries: Vec<Entry>,
    queries: HashMap<String, Container>,
}

#[cfg(test)]
impl Index {
    pub fn from_file(file: &[u8]) -> Index {
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

impl TryFrom<Bytes> for Index {
    type Error = &'static str;

    fn try_from(value: Bytes) -> Result<Self, Self::Error> {
        let mut value = value;

        let entries = {
            let size = value.get_u64();
            let bytes = value.split_to(size.try_into().unwrap());
            bincode::deserialize(bytes.as_ref()).unwrap()
        };

        let queries = {
            let size = value.get_u64();
            let bytes = value.split_to(size.try_into().unwrap());
            bincode::deserialize(bytes.as_ref()).unwrap()
        };

        Ok(Index { entries, queries })
    }
}

impl StorkIndex for Index {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::{BufReader, Read};

    #[test]
    fn can_parse_0_5_3_index() {
        let file = fs::File::open("../test-assets/federalist-min-0.5.3.st").unwrap();
        let mut buf_reader = BufReader::new(file);
        let mut index_bytes: Vec<u8> = Vec::new();
        let _bytes_read = buf_reader.read_to_end(&mut index_bytes);
        let index = Index::from_file(index_bytes.as_slice());
        assert_eq!(1, index.entries.len());
        assert_eq!(2477, index.queries.len());
    }

    #[test]
    fn can_parse_0_6_0_index() {
        let file = fs::File::open("../test-assets/federalist-min-0.6.0.st").unwrap();
        let mut buf_reader = BufReader::new(file);
        let mut index_bytes: Vec<u8> = Vec::new();
        let _bytes_read = buf_reader.read_to_end(&mut index_bytes);
        let index = Index::from_file(index_bytes.as_slice());
        assert_eq!(1, index.entries.len());
        assert_eq!(2477, index.queries.len());
    }
}
