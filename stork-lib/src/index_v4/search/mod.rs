use std::collections::HashMap;

use super::{Document, Excerpt, IndexDiskRepresentation, QueryResult};
use crate::{
    string_utils::{get_surroundings, segment_words},
    Document as OutputDocument, Excerpt as OutputExcerpt, HighlightRange, InternalWordAnnotation,
    Output, Result as OutputResult,
};

impl OutputExcerpt {
    fn from(excerpt: &Excerpt, document: &Document) -> Self {
        let (before, after, full) = {
            let (a, b, c) =
                get_surroundings(&document.contents, excerpt.contents_character_offset, 8);
            (a.unwrap(), b.unwrap(), c.unwrap())
        };

        let after_segmented = segment_words(after);

        OutputExcerpt {
            text: full.to_string(),
            highlight_ranges: vec![HighlightRange {
                beginning: before.len(),
                end: before.len() + after_segmented.first().unwrap().word.len(),
            }],
            score: 10,
            internal_annotations: vec![InternalWordAnnotation::Debug(format!(
                "char_offset: {}",
                excerpt.contents_character_offset
            ))],
            fields: HashMap::default(),
        }
    }
}

impl From<&Document> for OutputDocument {
    fn from(document: &Document) -> Self {
        Self {
            url: document.url.clone(),
            title: document.title.clone(),
            fields: HashMap::default(), // TODO
        }
    }
}

pub(crate) fn search(index: &IndexDiskRepresentation, query: &str) -> Output {
    let query_result_indices = index.query_tree.get_value_for_string(query);

    let mut excerpts_by_document: HashMap<OutputDocument, Vec<OutputExcerpt>> = HashMap::new();

    for query_result_index in query_result_indices {
        let query_result = index.query_results.get(query_result_index).unwrap();

        if let QueryResult::DocumentContentsExcerpt(excerpt) = query_result {
            let document = index.documents.get(excerpt.document_id).unwrap();
            let output_document: OutputDocument = document.into();
            let output_excerpt = OutputExcerpt::from(excerpt, document);

            excerpts_by_document
                .entry(output_document)
                .and_modify(|vec| vec.push(output_excerpt.clone()))
                .or_insert_with(|| vec![output_excerpt]);
        } else {
            panic!("Unhandled excerpt type when searching");
        }
    }

    let mut results: Vec<OutputResult> = excerpts_by_document
        .iter()
        .map(|(output_document, list_of_excerpts)| OutputResult {
            entry: output_document.clone(),
            excerpts: list_of_excerpts.clone(),
            title_highlight_ranges: vec![],
            score: 10 * list_of_excerpts.len(),
        })
        .collect();

    results.sort_by_key(|r| r.score);
    results.reverse();

    let total_hit_count = results.len();

    Output {
        results,
        total_hit_count,
        url_prefix: "".to_string(),
    }
}
