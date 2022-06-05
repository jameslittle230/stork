use crate::build::errors::DocumentReadError;
use crate::build::word_segmented_document::AnnotatedWord;
use crate::config::{Config, File};

use super::segment_words;

#[allow(clippy::unnecessary_wraps)]
pub(crate) fn generate(
    _config: &Config,
    _document_config: &File,
    contents: &str,
) -> Result<(String, Vec<AnnotatedWord>), DocumentReadError> {
    let word_list = segment_words(contents)
        .iter()
        .map(|indexed_word| AnnotatedWord {
            word: indexed_word.word.clone(),
            character_offset: indexed_word.character_offset,
            url_suffix: None,
        })
        .collect();
    Ok((contents.to_string(), word_list))
}
