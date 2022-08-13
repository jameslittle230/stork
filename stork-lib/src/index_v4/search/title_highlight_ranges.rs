use crate::{index_v4, search_output, string_utils};

pub(super) fn get(
    excerpt: &index_v4::TitleExcerpt,
    document: &index_v4::Document,
) -> Vec<search_output::HighlightRange> {
    let (before_offset, after_offset) = string_utils::get_words_surrounding_offset(
        &document.title,
        excerpt.title_character_offset,
        8,
    ); // TODO: Instead of 8, this should be infinity

    let after_segmented = string_utils::split_into_normalized_words(&after_offset);

    vec![search_output::HighlightRange {
        beginning: before_offset.len(),
        end: before_offset.len() + after_segmented.first().unwrap().word.len(),
    }]
}
