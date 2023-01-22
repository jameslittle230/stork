//! Contains a module for modeling a search query, as well as parsing strings into search query models.

// use serde::Serialize;

// use crate::index_v4::Settings;

use std::str::FromStr;

use itertools::Itertools;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SearchTerm {
    InexactWord(String),
    ExactWord(String),
    // ExactPhrase(String),
    // ExclusionTerm(String),
    MetadataFilter(String, String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchQuery {
    pub items: Vec<SearchTerm>,
}

impl FromStr for SearchQuery {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let items = s
            .split_ascii_whitespace() // TODO: Don't assume that word segmentation can be performed by splitting on whitespace
            .filter_map(|term| match term {
                term if term.contains('=') => {
                    if let Some((k, v)) = term.split_once('=') {
                        Some(SearchTerm::MetadataFilter(k.to_string(), v.to_string()))
                    } else {
                        None
                    }
                }

                term if term.starts_with('"') && term.ends_with('"') && term.len() > 4 => {
                    Some(SearchTerm::ExactWord(term.trim_matches('"').to_string()))
                }

                term if term.len() > 2 => Some(SearchTerm::InexactWord(term.to_string())),

                _ => None,
            })
            .collect_vec();

        Ok(SearchQuery { items })
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::{SearchQuery, SearchTerm};

    #[test]
    fn it_parses_multiple_inexact_words() {
        assert_eq!(
            "this is a big test".parse::<SearchQuery>(),
            Ok(SearchQuery {
                items: vec![
                    SearchTerm::InexactWord("this".to_string()),
                    SearchTerm::InexactWord("big".to_string()),
                    SearchTerm::InexactWord("test".to_string())
                ]
            })
        )
    }

    #[test]
    fn it_parses_a_single_inexact_word() {
        assert_eq!(
            "foobar".parse::<SearchQuery>(),
            Ok(SearchQuery {
                items: vec![SearchTerm::InexactWord("foobar".to_string())]
            })
        )
    }

    #[test]
    fn it_parses_a_single_exact_word() {
        assert_eq!(
            "\"foobar\"".parse::<SearchQuery>(),
            Ok(SearchQuery {
                items: vec![SearchTerm::ExactWord("foobar".to_string())]
            })
        )
    }

    #[test]
    fn it_parses_a_single_metadata_filter() {
        assert_eq!(
            "foo=bar".parse::<SearchQuery>(),
            Ok(SearchQuery {
                items: vec![SearchTerm::MetadataFilter(
                    "foo".to_string(),
                    "bar".to_string()
                )]
            })
        )
    }

    #[test]
    fn it_parses_an_empty_string() {
        assert_eq!("".parse::<SearchQuery>(), Ok(SearchQuery { items: vec![] }))
    }
}
