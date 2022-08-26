use std::collections::HashMap;

use crate::{search_output, stopwords};

use super::{Document, DocumentContentsExcerpt};

pub(super) fn build(
    excerpt: &DocumentContentsExcerpt,
    document: &Document,
    surrounding_word_count: u8,
    chars_remaining: u8,
) -> super::OutputExcerpt {
    let (before_offset, after_offset) = crate::string_utils::get_words_surrounding_offset(
        &document.contents,
        excerpt.contents_character_offset,
        surrounding_word_count.into(),
    );

    let after_segmented = crate::string_utils::split_into_normalized_words(&after_offset);
    let target_word = after_segmented.first().unwrap().word.clone();

    super::OutputExcerpt {
        text: before_offset.clone() + &after_offset,
        highlight_ranges: vec![search_output::HighlightRange {
            beginning: before_offset.len(),
            end: before_offset.len() + target_word.len(),
        }],
        score: match (
            stopwords::STOPWORDS.contains(&target_word.as_str()),
            chars_remaining,
        ) {
            (false, 0) => 55,
            (false, _) => 45_usize.saturating_sub(chars_remaining.into()),
            (true, 0) => 20,
            (true, _) => 20_usize.saturating_sub(chars_remaining.into()),
        },
        internal_annotations: vec![
            Some(super::InternalWordAnnotation::Debug(format!(
                "char_offset: {}",
                excerpt.contents_character_offset
            ))),
            excerpt
                .url_suffix
                .clone()
                .map(super::InternalWordAnnotation::UrlSuffix),
        ]
        .into_iter()
        .flatten()
        .collect(),
        fields: HashMap::default(),
    }
}
