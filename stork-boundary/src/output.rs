use serde::{Deserialize, Serialize};
use stork_shared::Fields;

#[derive(Serialize, Deserialize, Debug, Default, PartialEq)]
pub struct Output {
    pub results: Vec<Result>,
    pub total_hit_count: usize,
    pub url_prefix: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Result {
    pub entry: Entry,
    pub excerpts: Vec<Excerpt>,
    pub title_highlight_ranges: Vec<HighlightRange>,
    pub score: usize,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Entry {
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum InternalWordAnnotation {
    #[serde(rename = "a")]
    UrlSuffix(String),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct HighlightRange {
    pub beginning: usize,
    pub end: usize,
}

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct IndexMetadata {
    #[serde(rename = "indexVersion")]
    pub index_version: String,
}
