//! Errors that can occur during the index build process.

use thiserror::Error;

use crate::string_utils::pluralize_with_count;

use super::document_problem::AttributedDocumentProblem;

/// An error thrown when Stork has failed to build an index from the
/// given configuration file.
#[derive(Error, Debug)]
#[error("{error}")]
pub struct BuildError {
    error: InternalBuildError,
}

// TODO: Wire up all the display implementations correctly here
impl From<InternalBuildError> for BuildError {
    fn from(error: InternalBuildError) -> Self {
        Self { error }
    }
}

/// A build error is fatal.
#[derive(Debug, Error)]
pub(crate) enum InternalBuildError {
    // TODO: Make sure all these variants are being constructed in the right spot, and add tests for that
    #[error("No files specified in config.")]
    NoFilesSpecified,

    #[error("No files could be successfully read.")]
    AllDocumentsHadProblems(Vec<AttributedDocumentProblem>),

    /// Some users might choose to have any document error fail the entire build.
    /// If they configure this, their build will fail with this build error variant.
    #[error(
        "{} found while indexing files. If you want to fail silently and still build an index, remove `break_on_file_error` from your config.", 
        pluralize_with_count(.0.len(), "error", "errors"),
    )]
    SomeDocumentsHadProblems(Vec<AttributedDocumentProblem>),
}

impl PartialEq for InternalBuildError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::SomeDocumentsHadProblems(_), Self::SomeDocumentsHadProblems(_))
            | (Self::AllDocumentsHadProblems(_), Self::AllDocumentsHadProblems(_)) => true,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}
