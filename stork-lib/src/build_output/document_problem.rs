use thiserror::Error;

use crate::build::{
    parse_document::{html::HtmlParseError, srt::SRTParseError},
    read_contents::{file::FileReadError, url::UrlReadError},
};

/// A problem that came about when parsing or reading a single document. This
/// eventually becomes associated with a certain document and is stored as an
/// `AttributedDocumentProblem`.
#[derive(Debug, Error, Clone, PartialEq)]
pub(crate) enum DocumentProblem {
    // parse_document errors
    #[error("{0}")]
    SRTParseError(#[from] SRTParseError),

    #[error("{0}")]
    HtmlParseError(#[from] HtmlParseError),

    // read_contents errors
    #[error("{0}")]
    FileReadError(#[from] FileReadError),

    #[error("{0}")]
    WebDownloadError(#[from] UrlReadError),

    // Other errors
    #[error(
        "Cannot automatically figure out the filetype. Please give this file a file extension Stork knows about, or disambiguate the file\'s filetype within your config."
    )]
    CannotDetermineFiletype,

    #[error("After parsing the document, there were no words found in the word list.")]
    EmptyWordList,
}

/// Associates a `WordListGenerationError` with a `File`. A set of these might
/// be reported at the end of the build as `BuildWarning`s, or they might cause
/// the build to fail as part of a `BuildError`, depending on how the user has
/// configured their behavior.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct AttributedDocumentProblem {
    pub file_index: usize,
    pub problem: DocumentProblem,
}
