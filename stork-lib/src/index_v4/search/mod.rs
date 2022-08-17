mod build_output_excerpt;
mod title_highlight_ranges;

use std::collections::HashMap;

use super::{Document, DocumentContentsExcerpt, IndexDiskRepresentation, QueryResult};

use crate::{
    search_output::{Document as OutputDocument, Excerpt as OutputExcerpt, Result as OutputResult},
    string_utils,
};

use crate::search_output::{HighlightRange, InternalWordAnnotation, SearchResult};

impl From<&Document> for OutputDocument {
    fn from(document: &Document) -> Self {
        Self {
            url: document.url.clone(),
            title: document.title.clone(),
            fields: HashMap::default(), // TODO: Pass fields through
        }
    }
}

pub(crate) fn search(index: &IndexDiskRepresentation, query: &str) -> SearchResult {
    struct DocumentSearchOutput {
        contents_excerpts: Vec<OutputExcerpt>,
        title_highlight_ranges: Vec<HighlightRange>,
    }

    if let Some(query_result_indices) = index.query_tree.get_values_for_string(query) {
        let mut excerpts_by_document: HashMap<OutputDocument, DocumentSearchOutput> =
            HashMap::new();

        let mut last_resort_index: Option<crate::search_output::Excerpt> = None;

        for (categorization, query_result_index) in query_result_indices {
            let query_result = index.query_results.get(query_result_index).unwrap();

            match query_result {
                QueryResult::DocumentContentsExcerpt(excerpt) => {
                    let document = index.documents.get(&excerpt.document_id).unwrap();
                    let output_document: OutputDocument = document.into();
                    let output_excerpt = build_output_excerpt::build(
                        excerpt,
                        document,
                        index.settings.excerpt_buffer,
                        categorization,
                    );

                    excerpts_by_document
                        .entry(output_document)
                        .and_modify(|stuff| stuff.contents_excerpts.push(output_excerpt.clone()))
                        .or_insert_with(|| DocumentSearchOutput {
                            contents_excerpts: vec![output_excerpt],
                            title_highlight_ranges: vec![],
                        });
                }

                QueryResult::TitleExcerpt(excerpt) => {
                    let document = index.documents.get(&excerpt.document_id).unwrap();
                    let output_document: OutputDocument = document.into();
                    let mut title_highlight_ranges = title_highlight_ranges::get(excerpt, document);

                    excerpts_by_document
                        .entry(output_document)
                        .and_modify(|stuff| {
                            // TODO: Rename this from `stuff`
                            stuff
                                .title_highlight_ranges
                                .append(&mut title_highlight_ranges);
                        })
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

                QueryResult::MetadataValue(_) => panic!("Need to handle metadata value in search!"), // TODO
            }
        }

        let title_score_multiplier = match index.settings.title_boost {
            crate::config::TitleBoost::Minimal => 15,
            crate::config::TitleBoost::Moderate => 25,
            crate::config::TitleBoost::Large => 40,
            crate::config::TitleBoost::Ridiculous => 85,
        };

        let mut results: Vec<OutputResult> = excerpts_by_document
            .iter()
            .map(|(output_document, stuff)| {
                let mut excerpts = stuff.contents_excerpts.clone();
                excerpts.sort_by_key(|e| e.score);
                excerpts.reverse();
                excerpts.truncate(10);

                if let Some(last_resort) = last_resort_index.to_owned() {
                    if excerpts.is_empty() {
                        excerpts.push(last_resort)
                    }
                }

                OutputResult {
                    entry: output_document.clone(),
                    excerpts,
                    title_highlight_ranges: stuff.title_highlight_ranges.clone(),
                    score: 10 * stuff.contents_excerpts.len()
                        + title_score_multiplier * stuff.title_highlight_ranges.len(),
                }
            })
            .collect();

        results.sort_by_key(|r| r.score);
        results.reverse();

        let total_hit_count = results.len();

        SearchResult {
            results,
            total_hit_count,
            url_prefix: "".to_string(),
        }
    } else {
        SearchResult::default()
    }
}