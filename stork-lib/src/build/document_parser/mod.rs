use crate::config::{Config, File, Filetype};

use super::{
    contents_reader::ReadContentsOutput,
    errors::{AttributedDocumentReadError, DocumentReadError},
    word_segmented_document::WordSegmentedDocument,
};

mod html_word_list_generator;
mod markdown_word_list_generator;
mod plaintext_word_list_generator;
mod srt_word_list_generator;

pub(super) fn extract_document_contents(
    config: &Config,
    document_config: &File,
    ReadContentsOutput {
        contents, filetype, ..
    }: &ReadContentsOutput,
) -> Result<WordSegmentedDocument, AttributedDocumentReadError> {
    match filetype {
        Some(Filetype::PlainText) => {
            plaintext_word_list_generator::generate(config, document_config, contents)
        }
        Some(Filetype::SRTSubtitle) => {
            srt_word_list_generator::generate(config, document_config, contents)
        }
        Some(Filetype::HTML) => {
            html_word_list_generator::generate(config, document_config, contents)
        }
        Some(Filetype::Markdown) => {
            markdown_word_list_generator::generate(config, document_config, contents)
        }
        None => Err(DocumentReadError::CannotDetermineFiletype),
    }
    .map(|(contents, annotated_words)| WordSegmentedDocument {
        annotated_words,
        contents,
        stem_algorithm: document_config
            .stemming_override
            .clone()
            .unwrap_or(config.input.stemming.clone())
            .to_optional(),
        title: document_config.title.clone(),
        url: document_config.url.clone(),
    })
    .map_err(|e| AttributedDocumentReadError {
        document: document_config.clone(),
        read_error: e,
    })
}

#[cfg(test)]
mod tests {
    use crate::string_utils::segment_words;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_word_segmentation() {
        let output = segment_words("lorem ipsum");
        assert_eq!(output[1].character_offset, 6);
    }

    #[test]
    fn test_word_segmentation_multiple_spaces() {
        let output = segment_words("lorem  ipsum");
    }

    #[test]
    fn test_word_segmentation_spaces_dashes() {
        let output = segment_words("lorem  -- ipsum");
    }
}
