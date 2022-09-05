mod build_output_excerpt;
mod line_item;
mod title_highlight_ranges;

use std::collections::HashMap;

use super::{Document, DocumentContentsExcerpt, IndexDiskRepresentation, QueryResult};

use crate::{
    build_config::TitleBoost,
    search_output::{Document as OutputDocument, Excerpt as OutputExcerpt, Result as OutputResult},
    search_query,
    search_value::{SearchValue, V4SearchValue},
    string_utils,
};

use crate::search_output::{HighlightRange, InternalWordAnnotation, SearchResult};

fn title_score_multiplier(title_boost: &TitleBoost) -> usize {
    match title_boost {
        crate::build_config::TitleBoost::Minimal => 50,
        crate::build_config::TitleBoost::Moderate => 100,
        crate::build_config::TitleBoost::Large => 175,
        crate::build_config::TitleBoost::Ridiculous => 350,
    }
}

impl From<&Document> for OutputDocument {
    fn from(document: &Document) -> Self {
        Self {
            url: document.url.clone(),
            title: document.title.clone(),
            fields: HashMap::default(), // TODO: Pass fields through
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct ExtendedOutputExcerpt {
    output_excerpt: OutputExcerpt,
    document_contents_excerpt: DocumentContentsExcerpt,
}

#[derive(Debug, Clone, PartialEq)]
struct DocumentSearchOutput {
    contents_excerpts: Vec<ExtendedOutputExcerpt>,
    title_highlight_ranges: Vec<HighlightRange>,
}

pub(crate) fn get_search_values(
    index: &IndexDiskRepresentation,
    search_term: &search_query::SearchTerm,
) -> Vec<SearchValue> {
    let mut query_results = vec![];

    let values = match search_term {
        search_query::SearchTerm::InexactWord(word) => index
            .query_tree
            .get_values_for_string(word, super::tree::GetValuesOption::All),

        search_query::SearchTerm::ExactWord(word) => index
            .query_tree
            .get_values_for_string(word, super::tree::GetValuesOption::Exact),
        _ => panic!("TODO"),
    };

    if let Some(vec) = values {
        for (chars_remaining, result_index) in vec {
            if let Some(result) = index.query_results.get(result_index) {
                query_results.push(SearchValue {
                    v4_value: Some(crate::search_value::V4SearchValue {
                        result: result.clone(),
                        chars_remaining,
                    }),
                });
            }
        }
    }

    query_results
}

pub(crate) fn resolve_search_values(
    index: &IndexDiskRepresentation,
    search_values: Vec<SearchValue>,
) -> SearchResult {
    let mut per_document_search_output: HashMap<OutputDocument, DocumentSearchOutput> =
        HashMap::new();

    let mut last_resort_excerpts: HashMap<String, crate::search_output::Excerpt> = HashMap::new();

    for value in search_values {
        let V4SearchValue {
            result,
            chars_remaining,
        } = value.v4_value.unwrap();

        match result {
            QueryResult::DocumentContentsExcerpt(excerpt) => {
                let document = index.documents.get(&excerpt.document_id).unwrap();
                let output_document: OutputDocument = document.into();

                let output_excerpt = build_output_excerpt::build(
                    &excerpt,
                    document,
                    index.settings.excerpt_buffer,
                    chars_remaining, // The number of characters between the search term and the word's length
                );

                let extended_output_excerpt = ExtendedOutputExcerpt {
                    output_excerpt,
                    document_contents_excerpt: excerpt.clone(),
                };

                per_document_search_output
                    .entry(output_document)
                    .and_modify(
                        |DocumentSearchOutput {
                             contents_excerpts,
                             title_highlight_ranges: _,
                         }| {
                            contents_excerpts.push(extended_output_excerpt.clone())
                        },
                    )
                    .or_insert_with(|| DocumentSearchOutput {
                        contents_excerpts: vec![extended_output_excerpt.clone()],
                        title_highlight_ranges: vec![],
                    });
            }

            QueryResult::TitleExcerpt(excerpt) => {
                let document = index.documents.get(&excerpt.document_id).unwrap();
                let output_document: OutputDocument = document.into();
                let mut title_highlight_ranges = title_highlight_ranges::get(&excerpt, document);

                per_document_search_output
                    .entry(output_document)
                    .and_modify(
                        |DocumentSearchOutput {
                             contents_excerpts,
                             title_highlight_ranges: ranges,
                         }| {
                            ranges.append(&mut title_highlight_ranges);
                        },
                    )
                    .or_insert_with(|| DocumentSearchOutput {
                        contents_excerpts: vec![],
                        title_highlight_ranges,
                    });

                // TODO: Check to see if this document has any actual excerpts
                let _ = last_resort_excerpts
                    .entry(document.title.clone()) // TODO: Key this on an ID, not on the title, since titles can be identical
                    .or_insert_with(|| crate::search_output::Excerpt {
                        text: string_utils::get_words_surrounding_offset(
                            &document.contents,
                            0,
                            (index.settings.excerpt_buffer * 2).into(),
                        )
                        .1,
                        score: 50,
                        highlight_ranges: vec![],
                        internal_annotations: vec![],
                        fields: HashMap::new(),
                    });
            }

            QueryResult::MetadataValue(_) => {
                panic!("Need to handle metadata value in search!")
            } // TODO
        }
    }

    let title_score_multiplier = title_score_multiplier(&index.settings.title_boost);

    let mut results: Vec<OutputResult> = per_document_search_output
        .iter()
        .filter_map(
            |(
                output_document,
                DocumentSearchOutput {
                    contents_excerpts,
                    title_highlight_ranges,
                },
            )| {
                let mut excerpt_data_vec: Vec<line_item::SearchLineItem> = contents_excerpts
                    .iter()
                    .map(|ee| line_item::SearchLineItem {
                        text: ee.output_excerpt.text.clone(),
                        highlight_ranges: vec1::Vec1::try_from_vec(
                            // TODO: Are there any cases where this conversion fails?
                            ee.output_excerpt.highlight_ranges.clone(),
                        )
                        .unwrap(),
                        content_offset: ee.document_contents_excerpt.contents_character_offset,
                        score: ee.output_excerpt.score,
                        fields: ee.output_excerpt.fields.clone(),
                        internal_annotations: ee.output_excerpt.internal_annotations.clone(),
                        url_suffix: ee.document_contents_excerpt.url_suffix.clone(),
                    })
                    .collect();

                let mut excerpts: Vec<OutputExcerpt> =
                    line_item::merge::merge_all_excerpts(&mut excerpt_data_vec)
                        .iter()
                        .map(|data| OutputExcerpt {
                            text: data.text.clone(),
                            highlight_ranges: data.highlight_ranges.clone().into(),
                            score: data.score,
                            internal_annotations: data.internal_annotations.clone(),
                            fields: data.fields.clone(),
                        })
                        // .filter(|e| e.score > 20) // TODO: Don't filter if the query words are all stopwords
                        .collect();

                excerpts.sort_by_key(|e| e.score);
                excerpts.reverse();
                excerpts.truncate(10);

                if let Some(last_resort) = last_resort_excerpts.get(&output_document.title) {
                    if excerpts.is_empty() && !title_highlight_ranges.is_empty() {
                        excerpts.push(last_resort.clone())
                    }
                }

                // excerpts will be empty if the only excerpts were those for stopwords
                if excerpts.is_empty() {
                    return None;
                }

                // (highest individual score / 4) * (sqrt(number of matching excerpts))
                let excerpt_score_component: usize = ((excerpts.first().unwrap().score as f32
                    * 0.25)
                    * ((contents_excerpts.len() as f32).powf(0.5)))
                    as usize;

                let title_score_component = (title_score_multiplier as f32
                    * (title_highlight_ranges.len() as f32).powf(0.5))
                    as usize;

                let score = (excerpt_score_component * 10) + title_score_component;

                Some(OutputResult {
                    entry: output_document.clone(),
                    excerpts,
                    title_highlight_ranges: title_highlight_ranges.clone(),
                    score,
                })
            },
        )
        .collect();

    results.sort_by_key(|r| r.score);
    results.reverse();

    let total_hit_count = results.len(); // TODO: truncate

    SearchResult {
        results,
        total_hit_count,
        url_prefix: "".to_string(),
    }
}
