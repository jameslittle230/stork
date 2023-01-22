//! Data structures for displaying search results.

use crate::Fields;
use serde::Serialize;
use std::hash::{Hash, Hasher};

pub mod errors;

/// The set of data needed to display search results to a user.
#[derive(Serialize)]
#[cfg_attr(feature = "build", derive(Clone, Debug))]
pub struct SearchOutput {
    pub results: Vec<SearchResult>,
    pub total_hit_count: usize,
    pub url_prefix: String,
    pub query: String,
}

/// A single document in the list of matches for a search query, along with its
/// display information and excerpts.
#[derive(Serialize)]
#[cfg_attr(feature = "build", derive(Clone, Debug,))]
pub struct SearchResult {
    pub entry: Document,
    pub excerpts: Vec<Excerpt>,
    pub title_highlight_ranges: Vec<HighlightRange>,
    pub score: usize,
}

/// A document present in the search results.
#[derive(Serialize)]
#[cfg_attr(feature = "build", derive(Clone, Debug,))]
pub struct Document {
    pub url: String,
    pub title: String,
    pub fields: Fields,
}

impl PartialEq for Document {
    fn eq(&self, other: &Self) -> bool {
        self.url == other.url && self.title == other.title
    }
}

impl Hash for Document {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.url.hash(state);
        self.title.hash(state);
    }
}

/// An excerpt of a document's contents that contains words that were part
/// of the search query.
#[derive(Serialize)]
#[cfg_attr(feature = "build", derive(Clone, Debug))]
pub struct Excerpt {
    pub text: String,
    pub highlight_ranges: Vec<HighlightRange>,
    pub score: usize,
    pub url_suffix: Option<String>,
}

/// A range of characters in a string that should be highlighted.
/// The start and end indices are inclusive.
#[derive(Serialize)]
#[cfg_attr(feature = "build", derive(Clone, Debug))]
pub struct HighlightRange {
    pub beginning: usize,
    pub end: usize,
}
