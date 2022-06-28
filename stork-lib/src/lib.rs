#![warn(clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::must_use_candidate)]

use bytes::Bytes;
use lazy_static::lazy_static;

use std::collections::HashMap;
use std::convert::TryFrom;
use std::sync::Mutex;
use thiserror::Error;

pub type Fields = HashMap<String, String>;

mod output;
pub use output::{
    Entry, Excerpt, HighlightRange, IndexMetadata, InternalWordAnnotation, Output, Result,
};

mod input;
use input::{IndexVersioningError, VersionedIndex};

mod stopwords;
use stopwords::STOPWORDS as stopwords;

mod config;
pub use config::{Config, ConfigReadError};

mod string_utils;

#[cfg(feature = "search-v2")]
mod index_v2;
#[cfg(feature = "search-v3")]
mod index_v3;

mod index_v4;

#[cfg(feature = "search-v2")]
use {index_v2::search as V2Search, index_v2::Index as V2Index};

#[cfg(feature = "search-v3")]
use {index_v3::search as V3Search, index_v3::Index as V3Index};

#[cfg(feature = "build")]
mod build;

#[cfg(feature = "build")]
pub use build::{BuildError, BuildOutput, BuildWarning};

// We can't pass a parsed index over the WASM boundary so we store the parsed indices here
lazy_static! {
    static ref INDEX_CACHE: Mutex<HashMap<String, ParsedIndex>> = Mutex::new(HashMap::new());
}

/**
 * An error that may occur when trying to parse an index file.
 */
#[derive(Error, Debug)]
pub enum IndexParseError {
    #[error("{0}")]
    VersioningError(#[from] IndexVersioningError),

    #[error("Could not parse index, despite knowing the version.")]
    ParseError(),

    #[error("{0}")]
    V2Error(String),

    #[error("{0}")]
    V3Error(String),
}

#[derive(Debug)]
pub enum ParsedIndex {
    #[cfg(feature = "search-v2")]
    V2(V2Index),

    #[cfg(feature = "search-v3")]
    V3(V3Index),

    #[cfg(not(any(feature = "search-v2", feature = "search-v3")))]
    Unknown,
}

impl ParsedIndex {
    pub fn get_metadata(&self) -> IndexMetadata {
        match self {
            #[cfg(feature = "search-v2")]
            ParsedIndex::V2(_) => IndexMetadata {
                index_version: "stork-2".to_string(),
            },

            #[cfg(feature = "search-v3")]
            ParsedIndex::V3(_) => IndexMetadata {
                index_version: "stork-3".to_string(),
            },

            #[cfg(not(any(feature = "search-v2", feature = "search-v3")))]
            ParsedIndex::Unknown => IndexMetadata {
                index_version: "unknown".to_string(),
            },
        }
    }
}

#[allow(unreachable_patterns)]
pub fn index_from_bytes(bytes: Bytes) -> core::result::Result<ParsedIndex, IndexParseError> {
    let versioned = VersionedIndex::try_from(bytes)?;

    match versioned {
        #[cfg(feature = "search-v2")]
        VersionedIndex::V2(bytes) => V2Index::try_from(bytes)
            .map_err(|e| IndexParseError::V2Error(e.to_string()))
            .map(ParsedIndex::V2),

        #[cfg(feature = "search-v3")]
        VersionedIndex::V3(bytes) => V3Index::try_from(bytes)
            .map_err(|e| IndexParseError::V3Error(e.to_string()))
            .map(ParsedIndex::V3),

        _ => Err(IndexParseError::ParseError()),
    }
}
// #[derive(Debug, Error)]
// pub enum BuildError {
//     #[error("{0}")]
//     ConfigReadError(#[from] ConfigReadError),

//     #[error("The Stork binary was not compiled with the ability to build indexes. Please recompile with the `build_v3` feature enabled.")]
//     BinaryNotBuiltWithFeature,

//     #[error("{0}")]
//     #[cfg(feature = "build")]
//     IndexGenerationError(#[from] BuildError),
// }

// #[cfg(feature = "build")]
// impl From<&BuildOutput> for IndexDescription {
//     fn from(build_result: &BuildOutput) -> Self {
//         Self {
//             entries_count: build_result.entries_len(),
//             tokens_count: build_result.index.search_term_count(),
//             index_size_bytes: Bytes::from(&build_result.index).len(),
//             warnings: build_result.errors.clone(),
//         }
//     }
// }

#[cfg(not(feature = "build"))]
pub fn build_index(_config: &Config) -> core::result::Result<(), BuildError> {
    Err(BuildError::BinaryNotBuiltWithFeature)
}

#[cfg(feature = "build")]
pub fn build_index(config: &Config) -> core::result::Result<BuildOutput, BuildError> {
    build::build_index(config, Box::new(|progress| println!("{progress}"))) // TODO: Move this function to the CLI crate, just pass the closure through
}

pub fn register_index(
    name: &str,
    bytes: Bytes,
) -> core::result::Result<IndexMetadata, IndexParseError> {
    let parsed = index_from_bytes(bytes)?;
    // todo: save deserialized index to cache
    let metadata = parsed.get_metadata();
    INDEX_CACHE.lock().unwrap().insert(name.to_string(), parsed);
    Ok(metadata)
}

#[derive(Debug, Error)]
pub enum SearchError {
    #[error("{0}")]
    IndexParseError(#[from] IndexParseError),

    #[error("The index is not supported. You might need to recompile Stork with a different set of features enabled.")]
    IndexVersionNotSupported,

    #[error(
        "Index `{0}` has not been registered. You need to register the index before performing searches with it."
    )]
    IndexNotInCache(String),
}

pub fn search_from_cache(key: &str, query: &str) -> core::result::Result<Output, SearchError> {
    let cache = INDEX_CACHE.lock().unwrap();
    let parsed = match cache.get(key) {
        Some(parsed) => parsed,
        None => return Err(SearchError::IndexNotInCache(key.to_string())),
    };

    match parsed {
        #[cfg(feature = "search-v2")]
        ParsedIndex::V2(index) => Ok(V2Search(index, query)),

        #[cfg(feature = "search-v3")]
        ParsedIndex::V3(index) => Ok(V3Search(index, query)),

        #[cfg(not(any(feature = "search-v2", feature = "search-v3")))]
        ParsedIndex::Unknown => Err(SearchError::IndexVersionNotSupported),
    }
}

#[allow(unused_variables)]
pub fn search(index: Bytes, query: &str) -> core::result::Result<Output, SearchError> {
    let index = index_from_bytes(index)?;

    #[allow(unreachable_patterns)]
    match index {
        #[cfg(feature = "search-v3")]
        ParsedIndex::V3(index) => Ok(V3Search(&index, query)),

        #[cfg(feature = "search-v2")]
        ParsedIndex::V2(index) => Ok(V2Search(&index, query)),

        _ => Err(SearchError::IndexVersionNotSupported),
    }
}
