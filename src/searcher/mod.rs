use crate::index_versions::v2;
use crate::Fields;
use crate::IndexFromFile;
use serde::Serialize;
use std::convert::TryInto;

#[derive(Serialize)]
pub struct SearchOutput {
    pub results: Vec<OutputResult>,
    pub total_hit_count: usize,
}

#[derive(Serialize, Clone)]
pub struct OutputEntry {
    pub url: String,
    pub title: String,
    pub fields: Fields,
}

#[derive(Serialize, Clone)]
pub struct OutputResult {
    pub entry: OutputEntry,
    pub excerpts: Vec<Excerpt>,
    pub title_highlight_char_offset: Option<usize>,
}

#[derive(Serialize, Clone)]
pub struct Excerpt {
    pub text: String,
    pub highlight_char_offset: usize,
}

fn get_index_version(index: &IndexFromFile) -> String {
    let (version_size_bytes, rest) = index.split_at(std::mem::size_of::<u64>());
    let version_size = u64::from_be_bytes(version_size_bytes.try_into().unwrap());
    let (version_bytes, _rest) = rest.split_at(version_size as usize);
    String::from_utf8(version_bytes.to_vec()).unwrap()
}

pub fn search(index: &IndexFromFile, query: &str) -> SearchOutput {
    let version = get_index_version(index);
    let search_function = match version.as_str() {
        v2::VERSION_STRING => v2::search::search,
        _ => panic!("Unknown index version"),
    };

    search_function(index, query)
}
