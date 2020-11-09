mod common;
pub mod config;
pub mod wasm;
mod searcher;

mod index_versions;

use crate::searcher::parse::{IndexParseError, ParsedIndex};
use common::IndexFromFile;
use config::Config;
use searcher::SearchError;

pub use index_versions::v3 as LatestVersion;
use LatestVersion::builder;
use LatestVersion::builder::IndexGenerationError;
use LatestVersion::structs::Index;

use once_cell::sync::OnceCell;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::sync::Mutex;

// We can't pass a parsed index over the WASM boundary so we store the parsed indices here
static INDEX_CACHE: OnceCell<Mutex<HashMap<String, ParsedIndex>>> = OnceCell::new();

/**
 * Parses an index from a binary file and saves it in memory.
 */
pub fn parse_index(data: &IndexFromFile, name: &str) -> Result<ParsedIndex, IndexParseError> {
    let index = ParsedIndex::try_from(data)?;
    let mutex = INDEX_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let mut lock = mutex.lock().unwrap();
    lock.insert(name.to_string(), index.clone());
    Ok(index)
}

/**
 * Retrieves an index object from memory, and performs a search with the given query
 */
pub fn search(name: &str, query: &str) -> Result<searcher::SearchOutput, SearchError> {
    let parsed_indices = INDEX_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let lock = parsed_indices.lock().unwrap();
    let index = lock
        .get(name)
        .to_owned()
        .ok_or(SearchError::NamedIndexNotInCache)?;
    searcher::search(index, query)
}

/**
 * Builds an index that can be serialized and parsed later
 */
pub fn build(config: &Config) -> Result<Index, IndexGenerationError> {
    builder::build(config)
}
