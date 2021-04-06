use super::{ReadResult, ReaderConfig, WordListGenerationError};
use crate::{config::Filetype, LatestVersion::structs::AnnotatedWordList};

pub mod html_word_list_generator;
pub mod markdown_word_list_generator;
pub mod plaintext_word_list_generator;
pub mod srt_word_list_generator;

pub(super) fn create_word_list(
    config: &ReaderConfig,
    read_result: &ReadResult,
) -> Result<AnnotatedWordList, WordListGenerationError> {
    match read_result.filetype {
        Some(Filetype::PlainText) => plaintext_word_list_generator::generate(config, read_result),
        Some(Filetype::SRTSubtitle) => srt_word_list_generator::generate(config, read_result),
        Some(Filetype::HTML) => html_word_list_generator::generate(config, read_result),
        Some(Filetype::Markdown) => markdown_word_list_generator::generate(config, read_result),
        None => Err(WordListGenerationError::CannotDetermineFiletype),
    }
}
