use core::ops::{Add, Div};
use std::collections::HashMap;

use itertools::Itertools;
use serde::__private::doc;
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
        .map(|group| {
            // Called once per output excerpt, contains multiple "document excerpts", i.e. words

            // Get a sorted list of words that match the query

            #[rustfmt::skip]
            let first_offset = group.excerpts.first().unwrap().excerpt.contents_character_offset;

            #[rustfmt::skip]
            let last_offset = group.excerpts.last().unwrap().excerpt.contents_character_offset;

            let width = last_offset - first_offset;

            let buffer = (150 - width).div(2);

            let text_beginning_offset = first_offset.saturating_sub(buffer);
            let text_end_offset = std::cmp::min(last_offset.add(buffer), document.contents.len());
            // let width = last_offset - first_offset;

            let text = document.contents[text_beginning_offset..text_end_offset].to_string();

            let word_indices = text.unicode_word_indices().collect_vec();

            let mut word_indices_iter = word_indices.iter();

            // TODO: Chop off first word_index
            let highlight_ranges = group
                .excerpts
                .iter()
                .map(|excerpt| {
                    let word_index =
                        excerpt.excerpt.contents_character_offset - text_beginning_offset;
                    let (_, word) = word_indices_iter.find(|a| a.0 == word_index).unwrap();

                    HighlightRange {
                        beginning: word_index,
                        end: word_index + word.len(),
                    }
                })
                .collect_vec();

            let score = group.score();

            let url_suffix = group.excerpts.first().unwrap().excerpt.url_suffix.clone();

            Excerpt {
                text,
                highlight_ranges,
                score: score as usize,
                url_suffix,
            }
        })
        .collect_vec();

    excerpts.sort_by_key(|e| usize::MAX - e.score);
    // excerpts.truncate(10);

    SearchResult {
        entry: OutputDocument::from(document),
        excerpts,
        title_highlight_ranges: vec![],
        score: title_score_multiplier,
    }
}
