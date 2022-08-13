use crate::string_utils::split_into_normalized_words;

use super::annotated_word::AnnotatedWord;

#[allow(clippy::unnecessary_wraps)]
pub(crate) fn generate(
    _config: &crate::config::Config,
    _file_index: usize,
    contents: &str,
) -> (String, Vec<AnnotatedWord>) {
    let word_list = split_into_normalized_words(contents)
        .iter()
        .map(|indexed_word| {
            AnnotatedWord::new(indexed_word.word.clone(), indexed_word.byte_offset, None)
        })
        .collect();
    (contents.to_string(), word_list)
}
