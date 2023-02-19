use thiserror::Error;

use super::document_problem::AttributedDocumentProblem;

/// Build warnings are not fatal, but should be reported at the end of the
/// build process.
#[derive(Error, Debug, Clone)]
#[error(transparent)]
pub struct BuildWarning(#[from] BuildWarningRepr);

impl From<&AttributedDocumentProblem> for BuildWarning {
    fn from(value: &AttributedDocumentProblem) -> Self {
        Self::from(BuildWarningRepr::from(value.clone()))
    }
}

#[derive(Error, Debug, Clone, PartialEq)]
pub(crate) enum BuildWarningRepr {
    #[error("{0}")]
    DocumentProblem(#[from] AttributedDocumentProblem),
}
