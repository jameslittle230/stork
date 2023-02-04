use std::{
    collections::{HashMap, HashSet},
    ops::Div,
};

use itertools::Itertools;

use crate::{
    index_v4::DocumentId,
    search_output::{
        errors::SearchError, Document, Excerpt, HighlightRange, SearchOutput, SearchResult,
    },
    search_query::{MetadataFilter, SearchTerm},
    search_value,
};

use super::{tree::TreeRetrievalValue, ContentsExcerpt, Index, QueryResult, SearchValue};

// Use this method to get the search values for a given search term. Search values
// are merged together to create a `SearchOutput` struct, which contains data needed
// to render the search results for a given query.
//
// SearchValues can be cached by search term to make multi-word queries faster:
// an implementation only has to call this method once for each new search term,
// and merge new and cached values together.
pub(crate) fn get_search_values(
    index: &Index,
    term: &SearchTerm,
) -> Result<Vec<search_value::SearchValue>, SearchError> {
    let search_values = match term {
        SearchTerm::Inexact(inexact_word) => {
            let tree_values = index
                .query_result_tree
                .retrieve_values_for_string(inexact_word, false)
                .unwrap_or_default();

            tree_values
                .iter()
                .map(|tree_value| match tree_value {
                    TreeRetrievalValue::Value {
                        value,
                        characters_remaining,
                    } => SearchValue::InexactResult {
                        term: inexact_word.clone(),
                        result: value.clone(),
                        highlight_length: (inexact_word.len() as u8) + characters_remaining,
                        characters_remaining: *characters_remaining,
                    },
                    TreeRetrievalValue::UnloadedArena(_) => todo!(),
                })
                .collect_vec()
        }

        SearchTerm::Exact(exact_word) => {
            let tree_values = index
                .query_result_tree
                .retrieve_values_for_string(exact_word, true)
                .unwrap_or_default();

            tree_values
                .iter()
                .map(|tree_value| match tree_value {
                    TreeRetrievalValue::Value {
                        value,
                        characters_remaining: _,
                    } => SearchValue::ExactResult {
                        term: exact_word.clone(),
                        result: value.clone(),
                        highlight_length: (exact_word.len() as u8),
                    },
                    TreeRetrievalValue::UnloadedArena(_) => todo!(),
                })
                .collect_vec()
        }

        SearchTerm::MetadataFilter(metadata_filter) => {
            vec![SearchValue::Filter(metadata_filter.clone())]
        }
    };

    let output = search_values
        .iter()
        .map(|value| search_value::SearchValue {
            v4_value: Some(value.clone()),
        })
        .collect_vec();

    Ok(output)
}

#[allow(clippy::collapsible_match)]
pub(crate) fn render_search_values(
    index: &Index,
    search_values: Vec<SearchValue>,
    config: &crate::SearchConfig,
) -> Result<SearchOutput, SearchError> {
    type ValuesArrayIndex = usize;

    let mut query_results_by_document: HashMap<DocumentId, Vec<SearchValue>> = HashMap::new();
    let mut filters: Vec<MetadataFilter> = Vec::new();
    let mut results: Vec<SearchResult> = Vec::new();

    // Fill two previous data structures
    for value in search_values {
        match &value {
            SearchValue::ExactResult {
                term: _,
                result,
                highlight_length: _,
            } => {
                let document_id = match result {
                    QueryResult::ContentsExcerpt(contents_excerpt) => contents_excerpt.document_id,
                    QueryResult::TitleExcerpt(title_excerpt) => title_excerpt.document_id,
                };

                query_results_by_document
                    .entry(document_id)
                    .and_modify(|vec| vec.push((value).clone()))
                    .or_insert_with(|| vec![value.clone()]);
            }

            SearchValue::InexactResult {
                term: _,
                result,
                characters_remaining: _,
                highlight_length: _,
            } => {
                let document_id = match result {
                    QueryResult::ContentsExcerpt(contents_excerpt) => contents_excerpt.document_id,
                    QueryResult::TitleExcerpt(title_excerpt) => title_excerpt.document_id,
                };

                query_results_by_document
                    .entry(document_id)
                    .and_modify(|vec| vec.push(value.clone()))
                    .or_insert_with(|| vec![value.clone()]);
            }

            SearchValue::Filter(metadata_filter) => {
                filters.push(metadata_filter.clone());
            }
        }
    }

    for (document_id, query_results) in query_results_by_document {
        let document = index.documents.get(&document_id).unwrap();

        let mut contents_excerpts_with_hl = vec![];
        let mut title_excerpts_with_hl = vec![];

        query_results.iter().for_each(|value| match value {
            SearchValue::ExactResult {
                term,
                result,
                highlight_length: hl,
            } => match result {
                QueryResult::ContentsExcerpt(c) => contents_excerpts_with_hl.push((c, hl)),
                QueryResult::TitleExcerpt(t) => title_excerpts_with_hl.push((t, hl)),
            },

            SearchValue::InexactResult {
                term,
                result,
                characters_remaining: _,
                highlight_length: hl,
            } => match result {
                QueryResult::ContentsExcerpt(c) => contents_excerpts_with_hl.push((c, hl)),
                QueryResult::TitleExcerpt(t) => title_excerpts_with_hl.push((t, hl)),
            },

            SearchValue::Filter(_) => {
                unreachable!("query_results vec should have no filters in it")
            }
        });

        contents_excerpts_with_hl.sort_by_cached_key(|(ce, _)| ce.byte_offset);
        title_excerpts_with_hl.sort_by_cached_key(|(te, _)| te.byte_offset);

        let mut contents_excerpts_groupings: Vec<ContentExcerptGrouping> =
            contents_excerpts_with_hl.iter().fold(
                vec![],
                |mut accumulator, (contents_excerpt, highlight_length)| {
                    if let Some(last_grouping) = accumulator.last_mut() {
                        if last_grouping.can_swallow(contents_excerpt, config.excerpt_length) {
                            last_grouping.push(contents_excerpt, **highlight_length);
                            return accumulator;
                        }
                    }

                    accumulator.push(ContentExcerptGrouping::new(
                        contents_excerpt,
                        **highlight_length,
                    ));

                    accumulator
                },
            );

        // sort descending by score
        contents_excerpts_groupings.sort_by(|a, b| a.score().partial_cmp(&b.score()).unwrap());
        contents_excerpts_groupings.reverse();

        let document_score = contents_excerpts_groupings
            .iter()
            .fold(0.0, |acc, g| acc + g.score());

        contents_excerpts_groupings.truncate(config.number_of_excerpts);

        // ---- sum all grouping scores for each document to determine document scoring
        // ---- sort groupings by aggregated score
        // ---- create output excerpts for top n groupings
        let mut excerpts = contents_excerpts_groupings
            .iter()
            .map(|g| g.as_excerpt(document, config.excerpt_length))
            .collect_vec();

        excerpts.sort_by(|a, b| a.score.partial_cmp(&b.score).unwrap());
        excerpts.reverse();

        results.push(SearchResult {
            entry: document.into(),
            excerpts,
            title_highlight_ranges: vec![],
            score: document_score,
        })
    }

    let total_hit_count = results.len();
    results.sort_by(|a, b| a.score.partial_cmp(&b.score).unwrap());
    results.reverse();
    results.truncate(config.number_of_results);

    Ok(SearchOutput {
        results,
        total_hit_count,
        url_prefix: index.settings.url_prefix.clone(),
    })
}

type HighlightLength = u8;

struct ContentExcerptGrouping(Vec<(ContentsExcerpt, HighlightLength)>);

impl ContentExcerptGrouping {
    fn new(contents_excerpt: &ContentsExcerpt, highlight_length: HighlightLength) -> Self {
        Self(vec![(contents_excerpt.clone(), highlight_length)])
    }

    fn push(&mut self, contents_excerpt: &ContentsExcerpt, highlight_length: HighlightLength) {
        self.0.push((contents_excerpt.clone(), highlight_length));
    }

    fn can_swallow(&self, other: &ContentsExcerpt, excerpt_length: usize) -> bool {
        match (self.0.first(), self.0.last()) {
            (Some((first_excerpt_in_self, _)), Some((last_excerpt_in_self, _))) => {
                assert!(last_excerpt_in_self.byte_offset <= other.byte_offset);
                let diff = other.byte_offset - last_excerpt_in_self.byte_offset;
                diff < excerpt_length - 3
            }
            _ => unreachable!("Grouping should always have at least one element"),
        }
    }

    fn score(&self) -> f64 {
        let distinct_terms = self
            .0
            .iter()
            .fold(HashSet::new(), |mut acc, (excerpt, _)| {
                acc.insert(excerpt.importance.to_bits());
                acc
            })
            .len();

        let average_distance_between_words =
            self.last().byte_offset - self.first().byte_offset / self.len();

        let avg_modifier =
            distinct_terms as f64 / (((average_distance_between_words + 1) as f64).log2() + 1.0);

        let sum_of_importances = self
            .0
            .iter()
            .fold(0.0, |acc, (excerpt, _)| acc + excerpt.importance);

        sum_of_importances * avg_modifier * distinct_terms as f64
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn first(&self) -> &ContentsExcerpt {
        &self.0.first().unwrap().0
    }

    fn last(&self) -> &ContentsExcerpt {
        &self.0.last().unwrap().0
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn as_excerpt(&self, document: &super::Document, excerpt_length: usize) -> Excerpt {
        let first_byte = self.first().byte_offset.saturating_sub(
            (excerpt_length - 3)
                .saturating_sub(self.last().byte_offset - self.first().byte_offset)
                .div(2),
        ); // TODO: Trim to word bounds

        let last_byte = std::cmp::min(
            first_byte + excerpt_length,
            document.contents.first().unwrap().contents.len(),
        );

        let highlight_ranges = self
            .0
            .iter()
            .map(|(excerpt, hl)| HighlightRange {
                beginning: excerpt.byte_offset.saturating_sub(first_byte),
                end: excerpt.byte_offset.saturating_sub(first_byte) + (*hl as usize),
            })
            .collect_vec();

        Excerpt {
            text: document.contents.first().unwrap().contents[first_byte..last_byte].to_string(),
            highlight_ranges,
            score: self.score(),
            url_suffix: None,
        }
    }
}

impl From<&super::Document> for Document {
    fn from(value: &super::Document) -> Self {
        Self {
            url: value.url.clone(),
            title: value.title.clone(),
            fields: value.metadata.clone(),
        }
    }
}
