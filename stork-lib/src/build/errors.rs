use std::path::PathBuf;
use thiserror::Error;

use crate::config::File;
use crate::string_utils::pluralize_with_count;

/**
 * A problem that came about when parsing or reading a single document.
 * These might bubble up as a build *error* (failing the entire build), or they
 * might be aggregated and reported as build *warnings*.
 */
#[derive(Debug, Error, Clone, PartialEq)]
pub enum DocumentReadError {
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
    WebScrapingNotEnabled,
}

/**
 * A build error is fatal.
 */
#[derive(Debug, Error)]
pub enum BuildError {
    #[error("No files specified in config.")]
    NoFilesSpecified,

    #[error("No files could be successfully read.")]
    AllDocumentErrors(Vec<AttributedDocumentReadError>),

    /**
     * Some users might choose to have any document error fail the entire build.
     * If they configure this, their build will fail with this build error variant.
     */
    #[error(
        "{} found while indexing files. If you want to fail silently and still build an index, remove `break_on_file_error` from your config.", 
        pluralize_with_count(.0.len(), "error", "errors"),
    )]
    PartialDocumentErrors(Vec<AttributedDocumentReadError>),
    // TODO: These display implementations used to have the list of attributed errors,
    // but that's expensive. How can I use the `Display` feature to enable this?
    // Should it live here, or in the CLI crate?
}

impl PartialEq for BuildError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::PartialDocumentErrors(_), Self::PartialDocumentErrors(_))
            | (Self::AllDocumentErrors(_), Self::AllDocumentErrors(_)) => true,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

/**
 * Build warnings are not fatal, but should be reported at the end of the build
 * process.
 */
#[derive(Debug, Clone, PartialEq)]
pub enum BuildWarning {
    DocumentReadError(AttributedDocumentReadError),
    DeprecatedConfigSetting(DeprecatedConfigSetting),
}

/**
 * Associates a `WordListGenerationError` with a `File`. Can be part of a build warning
 * or build error.
 */
#[derive(Debug, Clone, PartialEq)]
pub struct AttributedDocumentReadError {
    pub document: File, // todo: does this mean I have to publish the File struct?
    pub read_error: DocumentReadError,
}

// fn display_attributed_document_read_error_list(errors: Vec<AttributedDocumentReadError>) -> String {
//     let mut error_list = String::new();
//     for error in errors {
//         let document = error.document;
//         let read_error = error.read_error;
//         error_list.push_str(&format!(r#""{file}": {read_error}\n"#));
//     }
//     error_list
// }

#[derive(Debug, Clone, PartialEq)]
pub enum DeprecatedConfigSetting {
    OutputDebugOption,
}
