//! Contains a module for modeling a search query, as well as parsing strings into search query models.

// use serde::Serialize;

// use crate::index_v4::Settings;

use std::str::FromStr;

use itertools::Itertools;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MetadataFilterComparator {
    Equals,
    NotEquals,
    GreaterThan,
    GreaterThanOrEquals,
    LessThan,
    LessThanOrEquals,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetadataFilter {
    key: String,
    value: String,
    comparator: MetadataFilterComparator,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SearchTerm {
    Inexact(String),
    Exact(String),
    MetadataFilter(MetadataFilter),
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
                        Some(SearchTerm::MetadataFilter(MetadataFilter {
                            key: k.to_string(),
                            value: v.to_string(),
                            comparator: MetadataFilterComparator::Equals,
                        }))
                    } else {
                        None
                    }
                }

                term if term.starts_with('"') && term.ends_with('"') && term.len() > 4 => {
                    Some(SearchTerm::Exact(term.trim_matches('"').to_string()))
                }

                term if term.len() > 2 => Some(SearchTerm::Inexact(term.to_string())),

                _ => None,
            })
            .collect_vec();

        Ok(SearchQuery { items })
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_parses_multiple_inexact_words() {
        assert_eq!(
            "this is a big test".parse::<SearchQuery>(),
            Ok(SearchQuery {
                items: vec![
                    SearchTerm::Inexact("this".to_string()),
                    SearchTerm::Inexact("big".to_string()),
                    SearchTerm::Inexact("test".to_string())
                ]
            })
        )
    }

    #[test]
    fn it_parses_a_single_inexact_word() {
        assert_eq!(
            "foobar".parse::<SearchQuery>(),
            Ok(SearchQuery {
                items: vec![SearchTerm::Inexact("foobar".to_string())]
            })
        )
    }

    #[test]
    fn it_parses_a_single_exact_word() {
        assert_eq!(
            "\"foobar\"".parse::<SearchQuery>(),
            Ok(SearchQuery {
                items: vec![SearchTerm::Exact("foobar".to_string())]
            })
        )
    }

    #[test]
    fn it_parses_a_single_metadata_filter() {
        assert_eq!(
            "foo=bar".parse::<SearchQuery>(),
            Ok(SearchQuery {
                items: vec![SearchTerm::MetadataFilter(MetadataFilter {
                    key: "foo".to_string(),
                    value: "bar".to_string(),
                    comparator: MetadataFilterComparator::Equals
                })]
            })
        )
    }

    #[test]
    fn it_parses_an_empty_string() {
        assert_eq!("".parse::<SearchQuery>(), Ok(SearchQuery { items: vec![] }))
    }
}
