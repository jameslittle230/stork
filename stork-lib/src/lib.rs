pub mod wasm;

use bytes::Bytes;

use once_cell::sync::OnceCell;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::sync::Mutex;

use stork_boundary::{IndexMetadata, IndexVersioningError, Output, VersionedIndex};
use stork_config::Config;
use stork_index_v2::search as V2Search;
use stork_index_v2::Index as V2Index;

use stork_index_v3::search as V3Search;
use stork_index_v3::Index as V3Index;

use thiserror::Error;

// We can't pass a parsed index over the WASM boundary so we store the parsed indices here
static INDEX_CACHE: OnceCell<Mutex<HashMap<String, ParsedIndex>>> = OnceCell::new();

#[derive(Error, Debug)]
pub enum IndexParseError {
    #[error("{0}")]
    VersioningError(#[from] IndexVersioningError),

    #[error("{0}")]
    V2Error(String),

    #[error("{0}")]
    V3Error(String),
}

#[derive(Error, Debug)]
pub enum SearchError {
    #[error("Index not found. You must parse an index before performing searches with it.")]
    NamedIndexNotInCache,
}

enum ParsedIndex {
    V2(V2Index),
    V3(V3Index),
}

fn index_from_data<'a>(data: Bytes) -> Result<ParsedIndex, IndexParseError> {
    let versioned = VersionedIndex::try_from(data)?;

    match versioned {
        VersionedIndex::V2(bytes) => V2Index::try_from(bytes)
            .map_err(|e| IndexParseError::V2Error(e.to_string()))
            .map(|index| ParsedIndex::V2(index)),

        VersionedIndex::V3(bytes) => V3Index::try_from(bytes)
            .map_err(|e| IndexParseError::V3Error(e.to_string()))
            .map(|index| ParsedIndex::V3(index)),
    }
}

/**
 * Parses an index from a binary file and saves it in memory.
 */
pub fn parse_and_cache_index(data: Bytes, name: &str) -> Result<IndexMetadata, IndexParseError> {
    let index = index_from_data(data)?;
    let index_metadata = IndexMetadata::from(&index);

    let mut hashmap = INDEX_CACHE
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .unwrap();
    hashmap.insert(name.to_string(), index);

    Ok(index_metadata)
}

/**
 * Retrieves an index object from memory, and performs a search with the given index binary and the given query.
 */
pub fn search_from_cache(name: &str, query: &str) -> Result<Output, SearchError> {
    let hashmap = INDEX_CACHE
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .unwrap();

    let index = hashmap
        .get(name)
        .to_owned()
        .ok_or(SearchError::NamedIndexNotInCache)?;

    match index {
        ParsedIndex::V3(inner) => Ok(V3Search(inner, query)),
        ParsedIndex::V2(inner) => Ok(V2Search(inner, query)),
    }
}

/**
 * Searches an Index object created from the `stork_search::build()` method.
 *
 * This method only works with indexes created with the same package version used to run the search.
 */
pub fn search_with_index(index: &V3Index, query: &str) -> Output {
    V3Search(index, query)
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
use stork_index_v3::IndexGenerationError;

#[cfg(feature = "build")]
pub fn build(config: &Config) -> Result<V3Index, IndexGenerationError> {
    let (index, _) = stork_index_v3::build(config)?;
    Ok(index)
}
