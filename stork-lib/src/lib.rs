#![allow(unused_variables)] // TODO: Remove and fix
#![allow(dead_code)] // TODO: Remove and fix

// #![warn(missing_docs)] // TODO: Uncomment and document when needed
// #![warn(clippy::pedantic)] // TODO: Uncomment and fix

pub mod build_output;
pub mod config;
pub mod parse_index;
pub mod search_output;

use bytes::Bytes;

mod build;
mod envelope;
mod fields;
mod index_v4;
mod stopwords;
mod string_utils;

// #[cfg(feature = "search-v2")]
// mod index_v2;

// #[cfg(feature = "search-v3")]
// mod index_v3;

#[cfg(not(feature = "build"))]
pub fn build_index(_config: &Config) -> core::result::Result<(), BuildError> {
    Err(BuildError::BinaryNotBuiltWithFeature)
}

/// Builds an index from a configuration struct.
///
/// ```
/// let config = config::from_file("./my_config.toml")
/// let index = build_index(config, None)
/// let search_results = search(&index, "second derivative")
/// ```
pub fn build_index(
    config: &config::Config,
    progress: Option<&dyn Fn(build_output::progress::Report)>,
) -> Result<build_output::success::Value, build_output::error::BuildError> {
    build::build_index(config, progress).map_err(build_output::error::BuildError::from)
}

/// Given some bytes, this function will try to unwrap it from its envelope and
/// parse it as a search index, returning that index if it's successful.
pub fn parse_bytes_as_index(
    bytes: Bytes,
) -> Result<parse_index::ParsedIndex, parse_index::IndexParseError> {
    parse_index::parse(bytes)
}

/// Some search indexes come in multiple chunks. If you've parsed the primary
/// chunk of an index and you have the bytes of a secondary chunk, you can use
/// this function to add that secondary data to your original index.
///
/// This function will mutate the index you pass in.
pub fn add_sidecar_bytes_to_index(
    index: &mut parse_index::ParsedIndex,
    bytes: Bytes,
) -> Result<(), parse_index::IndexParseError> {
    parse_index::add_sidecar_bytes_to_index(index, bytes)
}

/// Given a search index and a query, this function will return search results.
pub fn search(
    index: parse_index::ParsedIndex,
    query: &str,
) -> Result<search_output::SearchResult, search_output::error::SearchError> {
    // TODO: remove infallable from return result type
    match index.value {
        // parse_index::IndexType::V2Index(v2_index) => {
        //     return Err(errors::SearchError::NotCompiledWithFeature);
        //     // index_v2::search(&v2_index, query)
        // }
        // parse_index::IndexType::V3Index(v3_index) => {
        //     return Err(errors::SearchError::NotCompiledWithFeature);
        //     // index_v3::search(&v3_index, query)
        // }
        parse_index::IndexType::V4Index(v4_index) => Ok(index_v4::search(&v4_index, query)),
    }
}
