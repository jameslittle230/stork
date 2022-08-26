mod build_output_excerpt;
mod title_highlight_ranges;

use std::collections::HashMap;

use super::{Document, DocumentContentsExcerpt, IndexDiskRepresentation, QueryResult};

use crate::{
    config::TitleBoost,
    merge_search_results,
    search_output::{Document as OutputDocument, Excerpt as OutputExcerpt, Result as OutputResult},
    string_utils::{self, IndexedWord},
};

use crate::search_output::{HighlightRange, InternalWordAnnotation, SearchResult};

fn title_score_multiplier(title_boost: &TitleBoost) -> usize {
    match title_boost {
        crate::config::TitleBoost::Minimal => 50,
        crate::config::TitleBoost::Moderate => 100,
        crate::config::TitleBoost::Large => 175,
        crate::config::TitleBoost::Ridiculous => 350,
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

pub(crate) fn search(index: &IndexDiskRepresentation, query: &str) -> SearchResult {
    let query_words = string_utils::split_into_normalized_words(query);

    let mut per_document_search_output: HashMap<OutputDocument, DocumentSearchOutput> =
        HashMap::new();

    let mut last_resort_index: Option<crate::search_output::Excerpt> = None;

    for query_result_indices in
        query_words
            .into_iter()
            .filter_map(|IndexedWord { word, byte_offset }| {
                index.query_tree.get_values_for_string(&word)
            })
    {
        for (chars_remaining, query_result_index) in query_result_indices {
            let query_result = index.query_results.get(query_result_index).unwrap();

            match query_result {
                QueryResult::DocumentContentsExcerpt(excerpt) => {
                    let document = index.documents.get(&excerpt.document_id).unwrap();
                    let output_document: OutputDocument = document.into();

                    let output_excerpt = build_output_excerpt::build(
                        excerpt,
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
                    let mut title_highlight_ranges = title_highlight_ranges::get(excerpt, document);

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

                    if last_resort_index == None {
                        last_resort_index = Some(crate::search_output::Excerpt {
                            text: string_utils::get_words_surrounding_offset(
                                &document.contents,
                                0,
                                (index.settings.excerpt_buffer * 2).into(),
                            )
                            .1,
                            score: 0,
                            highlight_ranges: vec![],
                            internal_annotations: vec![],
                            fields: HashMap::new(),
                        })
                    }
                }

                QueryResult::MetadataValue(_) => {
                    panic!("Need to handle metadata value in search!")
                } // TODO
            }
        }
    }

    let title_score_multiplier = title_score_multiplier(&index.settings.title_boost);

    let mut results: Vec<OutputResult> = per_document_search_output
        .iter()
        .map(
            |(
                output_document,
                DocumentSearchOutput {
                    contents_excerpts,
                    title_highlight_ranges,
                },
            )| {
                let mut excerpt_data_vec: Vec<merge_search_results::ExcerptData> =
                    contents_excerpts
                        .iter()
                        .map(|ee| merge_search_results::ExcerptData {
                            text: ee.output_excerpt.text.clone(),
                            highlight_ranges: vec1::Vec1::try_from_vec(
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
                    merge_search_results::merge_all_excerpts(&mut excerpt_data_vec)
                        .iter()
                        .map(|data| OutputExcerpt {
                            text: data.text.clone(),
                            highlight_ranges: data.highlight_ranges.clone().into(),
                            score: data.score,
                            internal_annotations: data.internal_annotations.clone(),
                            fields: data.fields.clone(),
                        })
                        .collect();

                excerpts.sort_by_key(|e| e.score);
                excerpts.reverse();
                excerpts.truncate(10);

                // TODO: Does this fail if multiple documents need to use the last resort index? And shouldn't it be named `last_resort_excerpt` instead? I must have been tired while writing that.
                if let Some(last_resort) = last_resort_index.to_owned() {
                    if excerpts.is_empty() {
                        excerpts.push(last_resort)
                    }
                }

                // (highest individual score / 4) * (sqrt(number of matching excerpts))
                let excerpt_score_component: usize = ((excerpts.first().unwrap().score as f32
                    * 0.25)
                    * ((contents_excerpts.len() as f32).powf(0.5)))
                    as usize;

                let title_score_component = (title_score_multiplier as f32
                    * (title_highlight_ranges.len() as f32).powf(0.5))
                    as usize;

                if (title_score_component > 0) {
                    dbg!(excerpt_score_component, title_score_component);
                }

                let score = (excerpt_score_component * 10) + title_score_component;

                OutputResult {
                    entry: output_document.clone(),
                    excerpts,
                    title_highlight_ranges: title_highlight_ranges.clone(),
                    score,
                }
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
