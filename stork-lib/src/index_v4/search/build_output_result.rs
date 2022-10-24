use std::collections::HashMap;

use itertools::Itertools;
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    build_config::TitleBoost,
    index_v4::Document,
    search_output::{Document as OutputDocument, Excerpt, HighlightRange, SearchResult},
    string_utils,
};

use super::{
    bucket_search_values::TitleExcerptWithCharsRemaining,
    group_document_excerpts::GroupedDocumentExcerpts,
};

impl From<&Document> for OutputDocument {
    fn from(document: &Document) -> Self {
        Self {
            url: document.url.clone(),
            title: document.title.clone(),
            fields: HashMap::default(), // TODO: Pass fields through
        }
    }
}

fn title_score_multiplier(title_boost: &TitleBoost) -> usize {
    match title_boost {
        crate::build_config::TitleBoost::Minimal => 50,
        crate::build_config::TitleBoost::Moderate => 100,
        crate::build_config::TitleBoost::Large => 175,
        crate::build_config::TitleBoost::Ridiculous => 350,
    }
}

// Called once per document
pub(super) fn build_search_result(
    document: &Document,
    all_grouped_document_excerpts: Vec<GroupedDocumentExcerpts>,
    title_excerpts: &[TitleExcerptWithCharsRemaining],
    title_score_boost: &TitleBoost,
) -> SearchResult {
    let title_score_multiplier = title_score_multiplier(title_score_boost);

    let document_contents_word_indices = document.contents.unicode_word_indices().collect_vec();

    let mut excerpts = all_grouped_document_excerpts
        .iter()
        .map(|grouped_document_excerpts| {
            // Called once per output excerpt, contains multiple "document excerpts", i.e. words

            // Get a sorted list of words that match the query
            let mut document_excerpts = grouped_document_excerpts.excerpts.clone();
            document_excerpts.sort_by_key(|e| e.excerpt.contents_character_offset);
            let excerpt_character_offsets = document_excerpts
                .clone()
                .iter()
                .map(|excerpt| excerpt.excerpt.contents_character_offset)
                .collect_vec();

            let excerpt_word_indices_with_buffer = string_utils::get_text_excerpt(
                document_contents_word_indices.clone(),
                grouped_document_excerpts.starting_character_index,
                document_excerpts
                    .last()
                    .unwrap()
                    .excerpt
                    .contents_character_offset,
                120,
            );

            let text_beginning_offset = excerpt_word_indices_with_buffer.first().unwrap().0;
            let text_end_offset = excerpt_word_indices_with_buffer.last().unwrap().0
                + excerpt_word_indices_with_buffer.last().unwrap().1.len();
            let text = document.contents[text_beginning_offset..text_end_offset].to_string();

            let highlight_ranges = excerpt_word_indices_with_buffer
                .iter()
                .filter(|(offset, _)| excerpt_character_offsets.contains(offset))
                .map(|(offset, string)| {
                    assert!(
                        offset - text_beginning_offset + string.len() <= text.len(),
                        "{} {} ({text})",
                        offset - text_beginning_offset + string.len(),
                        text.len()
                    );
                    HighlightRange {
                        beginning: offset - text_beginning_offset,
                        end: offset - text_beginning_offset + string.len(),
                    }
                })
                .collect_vec();

            let score = grouped_document_excerpts.score;

            let url_suffix = document_excerpts
                .first()
                .unwrap()
                .excerpt
                .url_suffix
                .clone();

            Excerpt {
                text,
                highlight_ranges,
                score,
                url_suffix,
            }
        })
        .collect_vec();

    excerpts.sort_by_key(|e| usize::MAX - e.score);
    excerpts.truncate(10);

    SearchResult {
        entry: OutputDocument::from(document),
        excerpts,
        title_highlight_ranges: vec![],
        score: title_score_multiplier,
    }
}
