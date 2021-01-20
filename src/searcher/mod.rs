pub mod parse;
use parse::ParsedIndex;

use crate::common::{Fields, InternalWordAnnotation};
use crate::index_versions::{v2, v3};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Serialize, Deserialize, Debug, Default, PartialEq)]
pub struct SearchOutput {
    pub results: Vec<OutputResult>,
    pub total_hit_count: usize,
    pub url_prefix: String,
}

/**
 * Correlates an OutputEntry with a vector of excerpts. Represents a single
 * document that contains search results.
 */
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct OutputResult {
    pub entry: OutputEntry,
    pub excerpts: Vec<Excerpt>,
    pub title_highlight_ranges: Vec<HighlightRange>,
    pub score: usize,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct OutputEntry {
    pub url: String,
    pub title: String,
    pub fields: Fields,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Excerpt {
    pub text: String,
    pub highlight_ranges: Vec<HighlightRange>,
    pub score: usize,
    pub internal_annotations: Vec<InternalWordAnnotation>,
    pub fields: Fields,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct HighlightRange {
    pub beginning: usize,
    pub end: usize,
}

#[derive(Debug)]
pub enum SearchError {
    NamedIndexNotInCache,

    // If the JSON serialization engine crashes while turning the SearchOutput
    // into a string
    JSONSerializationError,

    // If there's a panic while performing the search (applicable to v2 only)
    InternalCrash,
}

impl fmt::Display for SearchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let desc: String = match self {
            SearchError::NamedIndexNotInCache => {
                "Index not found. You must parse an index before performing searches with it."
                    .to_string()
            }
            SearchError::JSONSerializationError => "Could not format search results.".to_string(),
            SearchError::InternalCrash => "Unknown error.".to_string(),
        };

        write!(f, "{}", desc)
    }
}

pub fn search(index: &ParsedIndex, query: &str) -> Result<SearchOutput, SearchError> {
    match index {
        ParsedIndex::V3(inner) => Ok(v3::search::search(inner, query)),
        ParsedIndex::V2(inner) => v2::search::search(inner, query),
    }
}
