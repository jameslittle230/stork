use crate::index_versions::v2;
use crate::Fields;
use crate::IndexFromFile;
use serde::Serialize;
use crate::index_analyzer::get_index_version;

#[derive(Serialize, Debug)]
pub struct SearchOutput {
    pub results: Vec<OutputResult>,
    pub total_hit_count: usize,
}

#[derive(Serialize, Clone, Debug)]
pub struct OutputEntry {
    pub url: String,
    pub title: String,
    pub fields: Fields,
}

#[derive(Serialize, Clone, Debug)]
pub struct OutputResult {
    pub entry: OutputEntry,
    pub excerpts: Vec<Excerpt>,
    pub title_highlight_char_offset: Option<usize>,
}

#[derive(Serialize, Clone, Debug)]
pub struct Excerpt {
    pub text: String,
    pub highlight_char_offset: usize,
}

pub fn search(index: &IndexFromFile, query: &str) -> SearchOutput {
    let version = get_index_version(index);
    let search_function = match version.as_str() {
        v2::VERSION_STRING => v2::search::search,
        _ => panic!("Unknown index version"),
    };

    search_function(index, query)
}
