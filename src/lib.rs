mod common;
pub mod config;
mod searcher;
pub mod wasm;

mod index_versions;

use crate::searcher::parse::{IndexParseError, ParsedIndex};
use common::IndexFromFile;
use config::Config;
use searcher::SearchError;

pub use index_versions::v3 as LatestVersion;
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
pub fn parse_and_cache_index(
    data: &IndexFromFile,
    name: &str,
) -> Result<ParsedIndex, IndexParseError> {
    let index = ParsedIndex::try_from(data)?;
    let mutex = INDEX_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let mut lock = mutex.lock().unwrap();
    lock.insert(name.to_string(), index.clone());
    Ok(index)
}

/**
 * Retrieves an index object from memory, and performs a search with the given index binary and the given query.
 */
pub fn search_from_cache(name: &str, query: &str) -> Result<searcher::SearchOutput, SearchError> {
    let parsed_indices = INDEX_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let lock = parsed_indices.lock().unwrap();
    let index = lock.get(name).ok_or(SearchError::NamedIndexNotInCache)?;
    searcher::search(index, query)
}

/**
 * Searches an Index object created from the `stork_search::build()` method.
 *
 * This method only works with indexes created with the same package version used to run the search.
 */
pub fn search_with_index(index: &Index, query: &str) -> searcher::SearchOutput {
    LatestVersion::search::search(index, query)
}

#[cfg(not(feature = "build"))]
pub fn build_index(_config: Option<&String>) -> (Config, Index) {
    println!("Stork was not compiled with support for building indexes. Rebuild the crate with default features to enable the test server.\nIf you don't expect to see this, file a bug: https://jil.im/storkbug\n");
    panic!()
}

/**
 * Builds an Index object that can be serialized and parsed later
 */
#[cfg(feature = "build")]
use LatestVersion::builder::errors::IndexGenerationError;

#[cfg(feature = "build")]
pub fn build(config: &Config) -> Result<Index, IndexGenerationError> {
    use LatestVersion::builder;
    let (index, _) = builder::build(config)?;
    Ok(index)
}
