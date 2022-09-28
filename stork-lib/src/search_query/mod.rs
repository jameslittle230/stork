//! Contains a module for modeling a search query, as well as parsing strings into search query models.

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum SearchTerm {
    InexactWord(String),
    ExactWord(String),
    ExactPhrase(String),
    // ExclusionTerm(String),
}
