use crate::index_versions::v3::{
    builder::annotated_words_from_string::AnnotatedWordable, structs::AnnotatedWordList,
};

use super::{ReadResult, ReaderConfig, WordListGenerationError};

pub fn generate(
    _config: &ReaderConfig,
    read_result: &ReadResult,
) -> Result<AnnotatedWordList, WordListGenerationError> {
    Ok(AnnotatedWordList {
        word_list: read_result.buffer.make_annotated_words(),
    })
}
