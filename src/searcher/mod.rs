pub mod index_analyzer;

use crate::common::{Fields, IndexFromFile, InternalWordAnnotation};
use crate::index_versions::{v2, v3};
use index_analyzer::{parse_index_version, IndexVersion};
use serde::Serialize;

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

pub struct SearchError {}

pub fn search(index: &IndexFromFile, query: &str) -> Result<SearchOutput, SearchError> {
    if let Ok(version) = parse_index_version(index) {
        let search_function = match version {
            IndexVersion::V2 => v2::search::search,
            IndexVersion::V3 => v3::search::search,
        };
        
        search_function(index, query)
    } else {
        Err(SearchError {})
    }
}
