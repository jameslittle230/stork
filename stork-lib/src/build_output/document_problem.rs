use std::fmt::Display;

use strunc::Strunc;
use thiserror::Error;

use crate::build::{
    parser::{html::HtmlParseError, srt::SRTParseError},
    reader::{file::FileReadError, url::UrlReadError},
};

/// A problem that came about when parsing or reading a single document. This
/// eventually becomes associated with a certain document and is stored as an
/// `AttributedDocumentProblem`.
#[derive(Error, Debug, Clone, PartialEq)]
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

    #[error("Document had contents from multiple sources. Please only use one of `contents`, `src_url`, or `path`.")]
    MultipleContentSources,
}

/// Associates a `WordListGenerationError` with a `File`. A set of these might
/// be reported at the end of the build as `BuildWarning`s, or they might cause
/// the build to fail as part of a `BuildError`, depending on how the user has
/// configured their behavior.
#[derive(Error, Debug, Clone, PartialEq)]
pub(crate) struct AttributedDocumentProblem {
    pub doc_title: String,
    pub problem: DocumentProblem,
}

impl Display for AttributedDocumentProblem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: {}",
            self.doc_title.strunc_len(25).to_string(),
            self.problem
        )
    }
}
