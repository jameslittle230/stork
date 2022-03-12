use crate::index_v3::{build::annotated_words_from_string::AnnotatedWordable, AnnotatedWordList};

use super::{ReadResult, ReaderConfig, WordListGenerationError};

#[allow(clippy::unnecessary_wraps)]
pub fn generate(
    _config: &ReaderConfig,
    read_result: &ReadResult,
) -> Result<AnnotatedWordList, WordListGenerationError> {
    Ok(AnnotatedWordList {
        word_list: read_result.buffer.make_annotated_words(),
    })
}
