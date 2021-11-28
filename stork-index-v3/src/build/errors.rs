use std::{fmt, path::PathBuf};
use stork_config::File;
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq)]
pub enum WordListGenerationError {
    #[error("SRT file could not be parsed.")]
    InvalidSRT,

    #[error("The file `{0}` could not be found.")]
    FileNotFound(PathBuf),

    #[error("Could not determine the file's filetype. Please give this file a file extension Stork knows about, or disambiguate the file's filetype within your config.")]
    CannotDetermineFiletype,

    #[error("The selector `{0}` is not present in the HTML document.")]
    SelectorNotPresent(String),

    #[error("The web page could not be fetched")]
    WebPageNotFetched,

    #[error("Content-Type is not present or invalid")]
    UnknownContentType,

    #[error("After parsing the document, there were no words found in the word list.")]
    EmptyWordList,
}

fn pluralize_with_count(count: usize, singular: &str, plural: &str) -> String {
    format!("{} {}", count, if count == 1 { singular } else { plural })
}

#[derive(Debug, Error)]
pub enum IndexGenerationError {
    #[error("No files specified in config file")]
    NoFilesSpecified,

    #[error("No files could be indexed")]
    NoValidFiles,

    #[allow(clippy::all)]
    #[error(
        "{} found while indexing files. If you want to fail silently and still build an index, remove `break_on_file_error` from your config.\n{}", 
        pluralize_with_count(.0.len(), "error", "errors"),
        DocumentError::display_list(.0)
    )]
    DocumentErrors(Vec<DocumentError>),
}

impl PartialEq for IndexGenerationError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::DocumentErrors(_), Self::DocumentErrors(_)) => true,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

/**
 * Associates a `WordListGenerationError` with a `File`.
 */
#[derive(Debug, Clone, PartialEq)]
pub struct DocumentError {
    pub file: File,
    pub word_list_generation_error: WordListGenerationError,
}

impl std::fmt::Display for DocumentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "- {}\n  {}",
            self.word_list_generation_error.to_string(),
            self.file
        )
    }
}

impl DocumentError {
    pub fn display_list(vec: &[DocumentError]) -> String {
        vec.iter()
            .map(ToString::to_string)
            .collect::<Vec<String>>()
            .join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_documenterrors_display() {
        let computed = DocumentError {
            file: File {
                title: "test".to_string(),
                ..Default::default()
            },
            word_list_generation_error: WordListGenerationError::FileNotFound(PathBuf::from(
                "/test",
            )),
        }
        .to_string();

        let expected = "- The file `/test` could not be found.\n  ";
        assert_eq!(computed, expected);
    }
}
