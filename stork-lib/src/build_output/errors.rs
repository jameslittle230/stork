//! Errors that can occur during the index build process.

use thiserror::Error;

use super::document_problem::AttributedDocumentProblem;

/// An error thrown when Stork has failed to build an index from the
/// given configuration file.
#[derive(Error, Debug)]
#[error(transparent)]
pub struct BuildError(#[from] BuildErrorRepr);

/// A build error is fatal.
#[derive(Error, Debug)]
pub(crate) enum BuildErrorRepr {
    // TODO: Make sure all these variants are being constructed in the right spot, and add tests for that
    #[error("No files specified in config.")]
    NoFilesSpecified,

    #[error("No files could be successfully read.")]
    AllDocumentsHadProblems(Vec<AttributedDocumentProblem>),

    /// Some users might choose to have any document error fail the entire build.
    /// If they configure this, their build will fail with this build error variant.
    #[error("Warning found while indexing files.")]
    OneDocumentHadProblem(AttributedDocumentProblem),
}

impl PartialEq for BuildErrorRepr {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::AllDocumentsHadProblems(_), Self::AllDocumentsHadProblems(_)) => true,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}
