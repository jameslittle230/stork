//! Contains a module for modeling a search query, as well as parsing strings into search query models.

use serde::Serialize;

use crate::index_v4::Settings;

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize)]
pub enum SearchTerm {
    InexactWord(String),
    ExactWord(String),
    // ExactPhrase(String),
    // ExclusionTerm(String),
    // MetadataFilter(String, String),
}

impl SearchTerm {
    pub(crate) fn is_valid(&self, settings: &Settings) -> bool {
        match self {
            SearchTerm::InexactWord(string) => {
                string.len() >= settings.minimum_query_length as usize
            }

            SearchTerm::ExactWord(string) => string.len() >= settings.minimum_query_length as usize,
        }
    }
}
