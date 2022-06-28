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

struct IndexedWord {
    word: String,
    character_offset: usize,
}

fn segment_words(string: &str) -> Vec<IndexedWord> {
    let split = string.split(|c: char| c.is_ascii_whitespace() || c == '-');

    let mut indexed_words: Vec<IndexedWord> = Vec::new();
    let mut offset_so_far = 0;

    for thing in split {
        if !thing.trim().is_empty() {
            indexed_words.push(IndexedWord {
                word: thing
                    .to_lowercase()
                    .trim_matches(|char: char| char.is_ascii_punctuation())
                    .to_string(),
                character_offset: offset_so_far,
            });
        }
        offset_so_far += thing.chars().count();
    }

    indexed_words
}

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
