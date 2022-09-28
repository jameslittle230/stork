#![allow(unused_variables)] // TODO: Remove and fix
#![allow(dead_code)]
// TODO: Remove and fix
// #![warn(missing_docs)]
// TODO: Uncomment and document when needed
// #![warn(clippy::pedantic)] // TODO: Uncomment and fix

//! The internal logic for building and searching [Stork](https://stork-search.net) search indexes.
//!
//! ```
//! let config =
//! let asdf = build_index()
//! ```

pub mod build_config;
pub mod build_output;
pub mod parse_index;
pub mod search_output;
pub mod search_query;
pub mod search_value;

use bytes::Bytes;

mod build;
mod envelope;
mod fields;
mod stopwords;
mod string_utils;

// #[cfg(feature = "search-v2")]
// mod index_v2;

#[cfg(feature = "search-v3")]
mod index_v3;

// #[cfg(feature = "search-v4")]
mod index_v4;

/// Builds an index from a configuration struct.
///
/// ```
/// let config = config::from_file("./my_config.toml")
/// let index = build_index(config, None)
/// let search_results = search(&index, "second derivative")
/// ```
pub fn build_index(
    config: &build_config::Config,
    progress: Option<&dyn Fn(build_output::ProgressReport)>,
) -> Result<build_output::BuildSuccessValue, build_output::errors::BuildError> {
    build::build_index(config, progress).map_err(build_output::errors::BuildError::from)
}

/// Given some bytes, this function will try to unwrap it from its envelope and
/// parse it as a search index, returning that index if it's successful.
pub fn parse_bytes_as_index(
    bytes: Bytes,
) -> Result<parse_index::ParsedIndex, parse_index::errors::IndexParseError> {
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
) -> Result<(), parse_index::errors::IndexParseError> {
    parse_index::add_sidecar_bytes_to_index(index, bytes)
}

/// Takes a string and parses it into a vector of `SearchTerm`s.
///
/// `SearchTerm`s can be used as cache keys for lists of `SearchValue`s.
pub fn parse_search_query_string(string: &str) -> Vec<search_query::SearchTerm> {
    string_utils::split_into_normalized_words(string)
        .iter()
        .map(|indexed_word| search_query::SearchTerm::InexactWord(indexed_word.word.clone()))
        .collect()
}

/// Given a `SearchTerm` and an index, this function returns a list of `SearchValue`s.
///
/// This list can be cached by the provided `SearchTerm` for faster search resolution.
pub fn get_search_values(
    index: &parse_index::ParsedIndex,
    term: &search_query::SearchTerm,
) -> Result<Vec<search_value::SearchValue>, search_output::errors::SearchError> {
    match &index.value {
        #[cfg(feature = "search-v3")]
        parse_index::IndexType::V3Index(_) => {
            Err(search_output::errors::SearchError::MethodNotAvailableForIndex)
        }
        parse_index::IndexType::V4Index(v4_index) => {
            Ok(index_v4::search::get_search_values(v4_index, term))
        }
    }
}

/// Given a lot of `SearchValue`s and an index, this function returns renderable search results.
pub fn merge_search_values(
    index: &parse_index::ParsedIndex,
    value_lists: Vec<Vec<search_value::SearchValue>>,
) -> Result<search_output::SearchResult, search_output::errors::SearchError> {
    let search_values: Vec<search_value::SearchValue> = value_lists.into_iter().flatten().collect();
    match &index.value {
        #[cfg(feature = "search-v3")]
        parse_index::IndexType::V3Index(_) => {
            Err(search_output::errors::SearchError::MethodNotAvailableForIndex)
        }
        parse_index::IndexType::V4Index(v4_index) => Ok(index_v4::search::resolve_search_values(
            v4_index,
            search_values,
        )),
    }
}

/// A helper method to perform a search. This method vends renderable search results
/// given an index and query string.
///
/// If you don't need to cache intermediate results, you can use this shorthand
/// `search` method which internally calls `parse_search_query_string`,
/// `get_search_values`, and `merge_line_items`.
pub fn search(
    index: &parse_index::ParsedIndex,
    query: &str,
) -> Result<search_output::SearchResult, search_output::errors::SearchError> {
    match &index.value {
        // parse_index::IndexType::V2Index(v2_index) => {
        //     return Err(errors::SearchError::NotCompiledWithFeature);
        //     // index_v2::search(&v2_index, query)
        // }
        #[cfg(feature = "search-v3")]
        parse_index::IndexType::V3Index(v3_index) => Ok(index_v3::search(v3_index, query)),

        parse_index::IndexType::V4Index(v4_index) => {
            let terms = parse_search_query_string(query);

            let values = terms
                .iter()
                .flat_map(|term| get_search_values(index, term).unwrap()) // TODO: Fix this unwrap
                .collect();

            Ok(index_v4::search::resolve_search_values(v4_index, values))
        }
    }
}
