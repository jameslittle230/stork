mod annotated_word;

use rust_stemmers::Algorithm as StemAlgorithm;

use super::read_contents::FileReadValue;
use crate::build_output::document_problem::{AttributedDocumentProblem, DocumentProblem};
use crate::{config::Filetype, fields::Fields, string_utils::split_into_normalized_words};

use annotated_word::AnnotatedWord;

pub(crate) mod html;
pub(crate) mod markdown;
pub(crate) mod plaintext;
pub(crate) mod srt;

/// The first layer of document processing. Holds the document's metadata and its
/// contents after those contents have gone through the word separation algorithm.
#[derive(Debug, Clone)]
pub(super) struct DocumentParseValue {
    pub(super) annotated_words: Vec<AnnotatedWord>,
    pub(super) contents: String, // The word separation algorithm is lossy - you're not guaranteed to be able to get the original contents back from the word list.
    pub(super) stem_algorithm: Option<StemAlgorithm>,
    pub(super) annotated_title_words: Vec<AnnotatedWord>,
    pub(super) title: String,
    pub(super) url: String,
    pub(super) fields: Option<Fields>,
}

pub(super) fn parse_document(
    config: &crate::config::Config,
    file_index: usize,
    FileReadValue {
        contents,
        filetype,
        frontmatter,
        ..
    }: &FileReadValue,
) -> Result<DocumentParseValue, AttributedDocumentProblem> {
    match filetype {
        Some(Filetype::PlainText) => Ok(plaintext::generate(config, file_index, contents)),
        Some(Filetype::SRTSubtitle) => {
            srt::generate(config, file_index, contents).map_err(DocumentProblem::from)
        }
        Some(Filetype::HTML) => {
            html::generate(config, file_index, contents).map_err(DocumentProblem::from)
        }
        Some(Filetype::Markdown) => {
            markdown::generate(config, file_index, contents).map_err(DocumentProblem::from)
        }
        None => Err(DocumentProblem::CannotDetermineFiletype), // Don't replace this with `_`
    }
    .map(|(contents, annotated_words)| {
        let document_config = config.input.files.get(file_index).unwrap();

        DocumentParseValue {
            annotated_words,
            contents: contents.replace('\n', " "),

            stem_algorithm: document_config
                .stemming_override
                .clone()
                .unwrap_or_else(|| config.input.stemming.clone())
                .to_optional(),

            annotated_title_words: split_into_normalized_words(&document_config.title)
                .iter()
                .map(|indexed_word| {
                    AnnotatedWord::new(indexed_word.word.clone(), indexed_word.byte_offset, None)
                })
                .collect(),

            title: document_config.title.clone(),
            url: document_config.url.clone(),
            fields: frontmatter.clone(),
        }
    })
    .map_err(|problem: DocumentProblem| AttributedDocumentProblem {
        file_index,
        problem,
    })
}

// TODO: Tests for the newline replacement
