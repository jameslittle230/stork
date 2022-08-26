//! Contains a module for modeling a search query, as well as parsing strings into search query models.

pub enum SearchTerm {
    InexactWord(String),
    ExactPhrase(String),
    // ExclusionTerm(String),
}
