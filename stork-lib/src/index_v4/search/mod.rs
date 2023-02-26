use std::collections::HashMap;

use itertools::Itertools;

use crate::{
    index_v4::{
        search::excerpt_grouping::{
            ContentsExcerptGrouping, ContentsExcerptWithHighlightLength,
            TitleExcerptWithHighlightLength,
        },
        DocumentId,
    },
    search_output::{errors::SearchError, Document, SearchOutput, SearchResult},
    search_query::{MetadataFilter, SearchTerm},
    search_value,
};

use super::{tree::TreeRetrievalValue, Index, QueryResult, SearchValue};

mod excerpt_grouping;

/**
Use this method to get the search values for a given search term. Search values
are merged together to create a `SearchOutput` struct, which contains data needed
to render the search results for a given query.

SearchValues can be cached by search term to make multi-word queries faster:
an implementation only has to call this method once for each new search term,
and merge new and cached values together.
*/
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

    // Groups the search values we want to render by the document to which that
    // search value belongs, so we can only consider an single document's search
    // values when rendering a `Result`.
    let mut query_results_by_document: HashMap<DocumentId, Vec<SearchValue>> = HashMap::new();

    let mut filters: Vec<MetadataFilter> = Vec::new();

    // Iterate over all search values to split them out and fill up
    // `query_results_by_document` and `filters` data structures
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

    // The list of output results that we'll render
    let mut results: Vec<SearchResult> = Vec::new();

    for (document_id, query_results) in query_results_by_document {
        let document = index.documents.get(&document_id).unwrap();

        let mut contents_excerpts = vec![];
        let mut title_excerpts = vec![];

        query_results.iter().for_each(|value| match value {
            SearchValue::ExactResult {
                term,
                result,
                highlight_length,
            } => match result {
                QueryResult::ContentsExcerpt(c) => {
                    contents_excerpts.push(ContentsExcerptWithHighlightLength {
                        contents_excerpt: c.clone(),
                        highlight_length: *highlight_length,
                        characters_remaining: 0,
                    })
                }
                QueryResult::TitleExcerpt(t) => {
                    title_excerpts.push(TitleExcerptWithHighlightLength {
                        title_excerpt: t.clone(),
                        highlight_length: *highlight_length,
                    })
                }
            },

            SearchValue::InexactResult {
                term,
                result,
                characters_remaining,
                highlight_length,
            } => match result {
                QueryResult::ContentsExcerpt(c) => {
                    contents_excerpts.push(ContentsExcerptWithHighlightLength {
                        contents_excerpt: c.clone(),
                        highlight_length: *highlight_length,
                        characters_remaining: *characters_remaining,
                    })
                }
                QueryResult::TitleExcerpt(t) => {
                    title_excerpts.push(TitleExcerptWithHighlightLength {
                        title_excerpt: t.clone(),
                        highlight_length: *highlight_length,
                    })
                }
            },

            SearchValue::Filter(_) => {
                unreachable!("query_results vec should have no filters in it")
            }
        });

        contents_excerpts.sort_by_cached_key(|ce| ce.contents_excerpt.byte_offset);
        title_excerpts.sort_by_cached_key(|te| te.title_excerpt.byte_offset);

        let mut contents_excerpts_groupings: Vec<ContentsExcerptGrouping> = contents_excerpts
            .iter()
            .fold(vec![], |mut accumulator, contents_excerpt| {
                if let Some(last_grouping) = accumulator.last_mut() {
                    if last_grouping.can_swallow(contents_excerpt, config.excerpt_length) {
                        last_grouping.push(contents_excerpt);
                        return accumulator;
                    }
                }

                accumulator.push(ContentsExcerptGrouping::new(contents_excerpt));
                accumulator
            });

        // sort descending by score
        contents_excerpts_groupings.sort_by(|a, b| a.score().partial_cmp(&b.score()).unwrap());
        contents_excerpts_groupings.reverse();

        let document_score = contents_excerpts_groupings
            .iter()
            .take(1)
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

impl From<&super::Document> for Document {
    fn from(value: &super::Document) -> Self {
        Self {
            url: value.url.clone(),
            title: value.title.clone(),
            fields: value.metadata.clone(),
        }
    }
}
