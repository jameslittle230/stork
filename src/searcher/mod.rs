pub mod index_analyzer;

use crate::common::{Fields, IndexFromFile, InternalWordAnnotation};
use crate::index_versions::{v2, v3};
use index_analyzer::{parse_index_version, IndexVersion, VersionParseError};
use serde::Serialize;
use std::fmt;

#[derive(Serialize, Debug, Default)]
pub struct SearchOutput {
    pub results: Vec<OutputResult>,
    pub total_hit_count: usize,
    pub url_prefix: String,
}

/**
 * Correlates an OutputEntry with a vector of excerpts. Represents a single
 * document that contains search results.
 */
#[derive(Serialize, Clone, Debug)]
pub struct OutputResult {
    pub entry: OutputEntry,
    pub excerpts: Vec<Excerpt>,
    pub title_highlight_ranges: Vec<HighlightRange>,
    pub score: usize,
}

#[derive(Serialize, Clone, Debug)]
pub struct OutputEntry {
    pub url: String,
    pub title: String,
    pub fields: Fields,
}

#[derive(Serialize, Clone, Debug)]
pub struct Excerpt {
    pub text: String,
    pub highlight_ranges: Vec<HighlightRange>,
    pub score: usize,
    pub internal_annotations: Vec<InternalWordAnnotation>,
    pub fields: Fields,
}

#[derive(Serialize, Clone, Debug)]
pub struct HighlightRange {
    pub beginning: usize,
    pub end: usize,
}

pub enum SearchError {
    /// If version can't be parsed when reading the index
    VersionParseError(VersionParseError),

    /// If the index deserialization returns an error (applicable to v3 only)
    IndexParseError(serde_cbor::error::Error),

    // If the JSON serialization engine crashes while turning the SearchOutput
    // into a string
    JSONSerializationError,

    // If there's a panic while performing the search (applicable to v2 only)
    InternalCrash,
}

impl fmt::Display for SearchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let desc: String = match self {
            SearchError::VersionParseError(e) => format!("{}", e),
            SearchError::IndexParseError(_e) => "Could not parse index file.".to_string(),
            SearchError::JSONSerializationError => "Could not format search results.".to_string(),
            SearchError::InternalCrash => "Unknown error.".to_string(),
        };

        write!(f, "{}", desc)
    }
}

pub fn search(index: &IndexFromFile, query: &str) -> Result<SearchOutput, SearchError> {
    match parse_index_version(index) {
        Ok(version) => {
            let search_function = match version {
                IndexVersion::V2 => v2::search::search,
                IndexVersion::V3 => v3::search::search,
            };

            search_function(index, query)
        }
        Err(e) => Err(SearchError::VersionParseError(e)),
    }
}
