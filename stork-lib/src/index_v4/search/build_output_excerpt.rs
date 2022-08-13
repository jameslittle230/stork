use std::collections::HashMap;

use crate::search_output;

pub(super) fn build(
    excerpt: &super::DocumentContentsExcerpt,
    document: &super::Document,
) -> super::OutputExcerpt {
    dbg!(&document.title);
    dbg!(excerpt); // TODO: Check that the document referenced during build is the same as the document referenced when searching
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
        score: 10,
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
