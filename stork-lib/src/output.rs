use crate::Fields;
use serde::{Deserialize, Serialize};

/**
 * The set of data needed to display search results to a user.
 */
#[derive(Serialize, Deserialize, Debug, Default, PartialEq)]
pub struct Output {
    pub results: Vec<Result>,
    pub total_hit_count: usize,
    pub url_prefix: String,
}

/**
 * A single document in the list of matches for a search query,
 * along with its display information and excerpts.
 */
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Result {
    pub entry: Entry,
    pub excerpts: Vec<Excerpt>,
    pub title_highlight_ranges: Vec<HighlightRange>,
    pub score: usize,
}

/**
 * A document present in the search results.
 */
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Entry {
    pub url: String,
    pub title: String,
    pub fields: Fields,
}

/**
 * An excerpt of a document's contents, that contains words that
 * were part of the search query.
 */
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Excerpt {
    pub text: String,
    pub highlight_ranges: Vec<HighlightRange>,
    pub score: usize,
    pub internal_annotations: Vec<InternalWordAnnotation>,
    pub fields: Fields,
}

/**
 * An annotation attached to a given excerpt.
 *
 * This should not be displayed directly to users, but instead should
 * change some aspect of how that excerpt is rendered.
 */
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum InternalWordAnnotation {
    #[serde(rename = "a")]
    UrlSuffix(String),
}

/**
 * A range of characters in a string that should be highlighted.
 * The start and end indices are inclusive.
 */
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct HighlightRange {
    pub beginning: usize,
    pub end: usize,
}

/**
 * Contains metadata about an index, to be displayed to the user, often for debugging.
 */
#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct IndexMetadata {
    #[serde(rename = "indexVersion")]
    pub index_version: String,
}
