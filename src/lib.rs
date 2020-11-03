mod common;
pub mod config;
pub mod searcher;
pub mod wasm;

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

pub fn parse_index(data: &IndexFromFile, name: &str) -> Result<ParsedIndex, IndexParseError> {
    let index = ParsedIndex::try_from(data)?;
    let mutex = INDEX_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let mut lock = mutex.lock().unwrap();
    lock.insert(name.to_string(), index.clone());
    Ok(index)
}

pub fn search(name: &str, query: &str) -> Result<searcher::SearchOutput, SearchError> {
    let parsed_indices = INDEX_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let lock = parsed_indices.lock().unwrap();
    let index = lock
        .get(name)
        .to_owned()
        .ok_or_else(|| SearchError::NamedIndexNotInCache)?;
    searcher::search(index, query)
}

pub fn build(config: &Config) -> Result<Index, IndexGenerationError> {
    builder::build(config)
}
