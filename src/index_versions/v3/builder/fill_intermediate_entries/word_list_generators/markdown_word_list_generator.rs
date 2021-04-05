use crate::LatestVersion::structs::AnnotatedWordList;

use super::{html_word_list_generator, ReadResult, ReaderConfig, WordListGenerationError};

pub fn generate(
    config: &ReaderConfig,
    read_result: &ReadResult,
) -> Result<AnnotatedWordList, WordListGenerationError> {
    let html_string = format!(
        "<html><body><main>{}</main></body></html>",
        markdown::to_html(&read_result.buffer)
    );

    let html_read_result = ReadResult {
        buffer: html_string,
        filetype: read_result.filetype.clone(),
        frontmatter_fields: None,
    };
    html_word_list_generator::generate(config, &html_read_result)
}
