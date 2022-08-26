//! A module responsible for making intermediate search results visible.
//!
//! The intent of exposing this module is to make it possible to cache the
//! `SearchLineItem`s for a single word, merging old cache values with newly-
//! computed values as the user types multiple words.

use vec1::Vec1;

use crate::{
    fields::Fields,
    search_output::{HighlightRange, InternalWordAnnotation},
};

pub(crate) mod merge;

/// An opaque data structure that contains a search result line item. A vector
/// of `SearchLineItem`s can be merged into a `SearchResult`.
#[derive(Debug, Clone, PartialEq)]
pub struct SearchLineItem {
    pub(crate) text: String,
    pub(crate) highlight_ranges: Vec1<HighlightRange>,
    pub(crate) content_offset: usize,
    pub(crate) score: usize,
    pub(crate) fields: Fields,
    pub(crate) internal_annotations: Vec<InternalWordAnnotation>,
    pub(crate) url_suffix: Option<String>,
}
