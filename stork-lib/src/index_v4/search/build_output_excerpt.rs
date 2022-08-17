use std::collections::HashMap;

use crate::search_output;

use super::{Document, DocumentContentsExcerpt};

pub(super) fn build(
    excerpt: &DocumentContentsExcerpt,
    document: &Document,
    chars_remaining: u8,
) -> super::OutputExcerpt {
    let (before_offset, after_offset) = crate::string_utils::get_words_surrounding_offset(
        &document.contents,
        excerpt.contents_character_offset,
        8, // TODO: Take the config value that the user passed in
    );

    let after_segmented = crate::string_utils::split_into_normalized_words(&after_offset);

    super::OutputExcerpt {
        text: before_offset.clone() + &after_offset,
        highlight_ranges: vec![search_output::HighlightRange {
            beginning: before_offset.len(),
            end: before_offset.len() + after_segmented.first().unwrap().word.len(),
        }],
        score: match chars_remaining {
            0 => 50,
            _ => 40_usize.saturating_sub(chars_remaining.into()),
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
