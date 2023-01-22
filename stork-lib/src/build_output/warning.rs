use super::document_problem::AttributedDocumentProblem;

/// Build warnings are not fatal, but should be reported at the end of
/// the build process.
pub struct BuildWarning {
    warning: BuildWarningInternal,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum BuildWarningInternal {
    DocumentProblem(AttributedDocumentProblem),
    DeprecatedConfigSetting(DeprecatedConfigSetting),
}

impl From<&AttributedDocumentProblem> for BuildWarning {
    fn from(problem: &AttributedDocumentProblem) -> Self {
        Self {
            warning: BuildWarningInternal::DocumentProblem(problem.clone()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeprecatedConfigSetting {}
