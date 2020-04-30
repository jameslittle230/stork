use crate::index_analyzer::get_index_version;
use crate::index_versions::v2;
use crate::index_versions::v3;
use crate::IndexFromFile;
use serde::Serialize;
use std::collections::HashMap;

type Fields = HashMap<String, String>;

#[derive(Serialize, Debug, Default)]
pub struct SearchOutput {
    pub results: Vec<OutputResult>,
    pub total_hit_count: usize,
    pub url_prefix: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct OutputEntry {
    pub url: String,
    pub title: String,
    pub fields: Fields,
}

/**
 * Correlates an OutputEntry with a vector of excerpts. Represents a single
 * document that contains search results.
 */
#[derive(Serialize, Clone, Debug)]
pub struct OutputResult {
    pub entry: OutputEntry,
    pub excerpts: Vec<Excerpt>,
    pub title_highlight_char_offset: Option<usize>,
    pub score: usize,
}

#[derive(Serialize, Clone, Debug)]
pub struct HighlightRange {
    pub beginning: usize,
    pub end: usize,
}

#[derive(Serialize, Clone, Debug)]
pub struct Excerpt {
    pub text: String,
    pub highlight_ranges: Vec<HighlightRange>,
    pub score: usize,
    pub fields: Fields,
}

pub fn search(index: &IndexFromFile, query: &str) -> SearchOutput {
    if let Ok(version) = get_index_version(index) {
        let search_function = match version.as_str() {
            v2::VERSION_STRING => v2::search::search,
            v3::VERSION_STRING => v3::search::search,
            _ => panic!("Unknown index version"),
        };

        search_function(index, query)
    } else {
        SearchOutput::default()
    }
}
