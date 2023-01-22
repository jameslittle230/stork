use std::{collections::HashMap, ops::Div};

use itertools::Itertools;

use crate::{
    index_v4::DocumentId,
    search_output::{self, Document, Excerpt, HighlightRange, SearchOutput, SearchResult},
    search_query::{self},
    search_value,
};

use super::{tree::TreeRetrievalValue, ContentsExcerpt, Index, QueryResult, V4SearchValue};

pub(crate) fn get_search_values(
    index: &Index,
    term: &search_query::SearchTerm,
) -> Result<Vec<search_value::SearchValue>, search_output::errors::SearchError> {
    let search_values = match term {
        search_query::SearchTerm::ExactWord(_) | search_query::SearchTerm::InexactWord(_) => {
            let values = match term {
                search_query::SearchTerm::InexactWord(inexact_word) => index
                    .query_result_tree
                    .retrieve_values_for_string(inexact_word, false)
                    .unwrap_or_default(),
                search_query::SearchTerm::ExactWord(exact_word) => index
                    .query_result_tree
                    .retrieve_values_for_string(exact_word, true)
                    .unwrap_or_default(),
                _ => unreachable!(),
            };

            values
                .iter()
                .filter_map(|value| {
                    Some(search_value::SearchValue {
                        v4_value: Some(V4SearchValue::SearchResult {
                            term: term.clone(),
                            result: value.clone(),
                        }),
                    })
                })
                .collect_vec()
        }

        search_query::SearchTerm::MetadataFilter(_, _) => vec![search_value::SearchValue {
            v4_value: Some(V4SearchValue::Filter { term: term.clone() }),
        }],
    };

    Ok(search_values)
}

pub(crate) fn merge_search_values(
    index: &Index,
    lists_of_values: Vec<Vec<search_value::SearchValue>>,
) -> Result<search_output::SearchOutput, search_output::errors::SearchError> {
    type ValuesArrayIndex = usize;

    // group values by document
    let mut values_for_document: HashMap<DocumentId, Vec<ValuesArrayIndex>> = HashMap::new();

    let values = lists_of_values
        .iter()
        .flatten()
        .filter_map(|v| match &v.v4_value {
            Some(V4SearchValue::Filter { term }) => todo!(),
            Some(V4SearchValue::SearchResult { term, result }) => return Some(result),
            None => return None,
        })
        .collect_vec();

    values.iter().enumerate().for_each(|(index, value)| {
        let document_id = match value {
            TreeRetrievalValue::Value {
                value,
                characters_remaining,
            } => match value {
                QueryResult::ContentsExcerpt(contents_excerpt) => contents_excerpt.document_id,
                QueryResult::TitleExcerpt(title_excerpt) => title_excerpt.document_id,
            },
            TreeRetrievalValue::UnloadedArena(_) => todo!(),
        };

        values_for_document
            .entry(document_id)
            .and_modify(|vec| vec.push(index))
            .or_insert_with(|| vec![index]);
    });

    // for each document:
    let mut results = values_for_document
        .iter()
        .map(|(document_id, value_indices)| {
            let mut sorted_value_indices = value_indices.clone();
            sorted_value_indices.sort_by_cached_key(|idx| {
                extract_contents_excerpt(&values[*idx])
                    .map(|contents_excerpt| contents_excerpt.byte_offset)
                    .unwrap_or(0)
            });

            // ---- create and score groupings
            let mut groupings: Vec<ContentExcerptGrouping> = sorted_value_indices
                .iter()
                .filter_map(|value_index| {
                    let value = &values[*value_index];
                    match value {
                        TreeRetrievalValue::Value {
                            value,
                            characters_remaining,
                        } => match value {
                            QueryResult::ContentsExcerpt(contents_excerpt) => {
                                Some((value_index, contents_excerpt))
                            }
                            QueryResult::TitleExcerpt(_) => None,
                        },
                        TreeRetrievalValue::UnloadedArena(_) => None,
                    }
                })
                .fold(
                    vec![],
                    |mut accumulator, (value_index, contents_excerpt)| {
                        if let Some(last_grouping) = accumulator.last_mut() {
                            if let TreeRetrievalValue::Value {
                                value: prev_query_result,
                                characters_remaining: _, // TODO: Incorporate into scoring
                            } = &values[*last_grouping
                                .value_indices
                                .first()
                                .expect("Vec expected to always have >=1 value")]
                            {
                                if let QueryResult::ContentsExcerpt(prev_contents_excerpt) =
                                    prev_query_result
                                {
                                    let diff = &contents_excerpt.byte_offset
                                        - prev_contents_excerpt.byte_offset;

                                    if diff < 150 - 3 {
                                        // Adding onto the existing grouping
                                        let diff_score = 150 - diff;
                                        last_grouping.value_indices.push(*value_index);

                                        // TODO: Don't score groupings iteratively; instead, aggregate data in the ContentExcerptGrouping and calculate a score after the fact.
                                        // This will let me add the "how many unique words are there" metric to the scoring algo.
                                        last_grouping.score +=
                                            contents_excerpt.importance + diff_score as f64;
                                    } else {
                                        // Starting a new grouping
                                        accumulator.push(ContentExcerptGrouping {
                                            value_indices: vec![*value_index],
                                            score: contents_excerpt.importance,
                                        });
                                    }
                                }
                            }
                        } else {
                            // Once per document, starting the list of groupings
                            accumulator.push(ContentExcerptGrouping {
                                value_indices: vec![*value_index],
                                score: contents_excerpt.importance,
                            });
                        }

                        accumulator
                    },
                );

            groupings.sort_by_key(|g| i32::MAX - g.score as i32);

            let document_score = groupings.iter().fold(0.0, |acc, g| acc + g.score);

            groupings.truncate(10);

            let document = index.documents.get(document_id).unwrap();

            // ---- sum all grouping scores for each document to determine document scoring
            // ---- sort groupings by aggregated score
            // ---- create output excerpts for top n groupings
            let mut excerpts = groupings
                .iter()
                .map(|g| g.into_excerpt(&values, document))
                .collect_vec();

            excerpts.sort_by_key(|e| e.score);
            excerpts.reverse();

            SearchResult {
                entry: index.documents.get(document_id).unwrap().into(),
                excerpts,
                title_highlight_ranges: vec![],
                score: document_score as usize,
            }
        })
        .collect_vec();
    // create output results for top m documents

    let total_hit_count = results.len();
    results.sort_by_key(|r| r.score);
    results.reverse();
    results.truncate(10);

    Ok(SearchOutput {
        results,
        total_hit_count,
        url_prefix: index.settings.url_prefix.clone(),
        query: "".to_string(), // TODO
    })
}

fn extract_contents_excerpt(
    tree_value: &TreeRetrievalValue<QueryResult>,
) -> Option<ContentsExcerpt> {
    match tree_value {
        TreeRetrievalValue::Value {
            value,
            characters_remaining,
        } => match value {
            QueryResult::ContentsExcerpt(contents_excerpt) => Some(contents_excerpt.clone()),
            QueryResult::TitleExcerpt(_) => None,
        },
        TreeRetrievalValue::UnloadedArena(_) => None,
    }
}

struct ContentExcerptGrouping {
    value_indices: Vec<usize>,
    score: f64,
}

impl ContentExcerptGrouping {
    fn into_excerpt(
        &self,
        values: &Vec<&TreeRetrievalValue<QueryResult>>,
        document: &super::Document,
    ) -> Excerpt {
        let excerpts = self
            .value_indices
            .iter()
            .map(|idx| extract_contents_excerpt(&values[*idx]).unwrap())
            .collect_vec();

        let first_byte = excerpts.first().unwrap().byte_offset.saturating_sub(
            147_usize
                .saturating_sub(
                    excerpts.last().unwrap().byte_offset - excerpts.first().unwrap().byte_offset,
                )
                .div(2),
        ); // TODO: Trim to word bounds

        let last_byte = std::cmp::min(
            first_byte + 150,
            document.contents.first().unwrap().contents.len(),
        );

        let highlight_ranges = excerpts
            .iter()
            .map(|excerpt| HighlightRange {
                beginning: excerpt.byte_offset.saturating_sub(first_byte),
                end: excerpt.byte_offset.saturating_sub(first_byte) + 3, // TODO: Feed through word length
            })
            .collect_vec();

        Excerpt {
            text: document.contents.first().unwrap().contents[first_byte..last_byte].to_string(),
            highlight_ranges,
            score: self.score as usize,
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
