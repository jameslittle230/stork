use std::{fmt, path::PathBuf};
use thiserror::Error;

use crate::config::File;

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

    #[error("When fetched, the web page returned a {0} status code.")]
    WebPageErrorfulStatusCode(u16),

    #[error("Content-Type is not present or invalid")]
    UnknownContentType,

    #[error("After parsing the document, there were no words found in the word list.")]
    EmptyWordList,

    #[error("Stork was not built with the `web-scraping` feature enabled.")]
    FeatureNotAvailable,
}

fn pluralize_with_count(count: usize, singular: &str, plural: &str) -> String {
    format!("{count} {}", if count == 1 { singular } else { plural })
}

#[derive(Debug, Error)]
pub enum IndexGenerationError {
    #[error("No files specified in config file")]
    NoFilesSpecified,

    #[error("All files failed to be indexed.\n{}", DocumentError::display_list(.0))]
    AllDocumentErrors(Vec<DocumentError>),

    #[error(
        "{} found while indexing files. If you want to fail silently and still build an index, remove `break_on_file_error` from your config.\n{}", 
        pluralize_with_count(.0.len(), "error", "errors"),
        DocumentError::display_list(.0)
    )]
    PartialDocumentErrors(Vec<DocumentError>),
}

impl PartialEq for IndexGenerationError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::PartialDocumentErrors(_), Self::PartialDocumentErrors(_))
            | (Self::AllDocumentErrors(_), Self::AllDocumentErrors(_)) => true,
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
            "In file `{}`: {}",
            self.file, self.word_list_generation_error,
        )
    }
}

impl DocumentError {
    #[must_use]
    pub fn display_list(vec: &[DocumentError]) -> String {
        format!(
            "Warning: Stork couldn't include {} in the index because of the following errors:\n",
            pluralize_with_count(vec.len(), "file", "files"),
        ) + &vec
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<String>>()
            .join("\n")
    }
}

#[cfg(test)]
mod tests {
    use crate::config::DataSource;

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_documenterrors_display() {
        let computed = DocumentError {
            file: File {
                title: "My Test File".to_string(),
                explicit_source: Some(DataSource::Contents("ignored".to_string())),
                ..File::default()
            },
            word_list_generation_error: WordListGenerationError::FileNotFound(PathBuf::from(
                "/test",
            )),
        }
        .to_string();

        let expected = "In file `My Test File`: The file `/test` could not be found.";
        assert_eq!(computed, expected);
    }

    #[test]
    fn test_documenterror_list_display() {
        let computed = DocumentError::display_list(&[
            DocumentError {
                file: File {
                    title: "My Test File".to_string(),
                    explicit_source: Some(DataSource::Contents("ignored".to_string())),
                    ..File::default()
                },
                word_list_generation_error: WordListGenerationError::FileNotFound(PathBuf::from(
                    "/test",
                )),
            },
            DocumentError {
                file: File {
                    title: "My Test File 2".to_string(),
                    explicit_source: Some(DataSource::Contents("ignored 2".to_string())),
                    ..File::default()
                },
                word_list_generation_error: WordListGenerationError::FileNotFound(PathBuf::from(
                    "/test2",
                )),
            },
        ]);

        let expected = "Warning: Stork couldn't include 2 files in the index because of the following errors:\nIn file `My Test File`: The file `/test` could not be found.\nIn file `My Test File 2`: The file `/test2` could not be found.";
        assert_eq!(computed, expected);
    }
}
