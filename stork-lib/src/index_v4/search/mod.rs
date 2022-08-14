mod build_output_excerpt;
mod title_highlight_ranges;

use std::collections::HashMap;

use super::{Document, DocumentContentsExcerpt, IndexDiskRepresentation, QueryResult};

use crate::search_output::{
    Document as OutputDocument, Excerpt as OutputExcerpt, Result as OutputResult,
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

    let query_result_indices = index.query_tree.get_values_for_string(query);

    dbg!(query_result_indices);

    let mut excerpts_by_document: HashMap<OutputDocument, DocumentSearchOutput> = HashMap::new();

    for query_result_index in query_result_indices {
        let query_result = index.query_results.get(query_result_index).unwrap();

        match query_result {
            QueryResult::DocumentContentsExcerpt(excerpt) => {
                let document = index.documents.get(&excerpt.document_id).unwrap();
                let output_document: OutputDocument = document.into();
                let output_excerpt = build_output_excerpt::build(excerpt, document);

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
            }

            QueryResult::MetadataValue(_) => panic!("Need to handle metadata value in search!"), // TODO
        }
    }

    let mut results: Vec<OutputResult> = excerpts_by_document
        .iter()
        .map(|(output_document, stuff)| OutputResult {
            entry: output_document.clone(),
            excerpts: stuff.contents_excerpts.clone(),
            title_highlight_ranges: stuff.title_highlight_ranges.clone(),
            score: 10 * stuff.contents_excerpts.len() + 30 * stuff.title_highlight_ranges.len(), // TODO: Tweak scores, use the title boost config
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
}
