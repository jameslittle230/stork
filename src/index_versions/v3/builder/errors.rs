use crate::config::File;
use colored::Colorize;
use std::{error::Error, fmt};

#[derive(Debug, Clone, PartialEq)]
pub enum WordListGenerationError {
    InvalidSRT,
    FileNotFound,
    CannotDetermineFiletype,
    SelectorNotPresent(String),
    WebPageNotFetched,
    UnknownContentType,
    EmptyWordList,
}

impl fmt::Display for WordListGenerationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let desc: String = match self {
            WordListGenerationError::InvalidSRT => "SRT file could not be parsed".to_string(),
            WordListGenerationError::SelectorNotPresent(selector_string) => format!(
                "HTML selector `{}` is not present in the file",
                selector_string
            ),
            WordListGenerationError::FileNotFound => "The file could not be found".to_string(),
            WordListGenerationError::CannotDetermineFiletype => "Could not determine the filetype. Please use a known file extension or disambiguate the filetype within your configuration file".to_string(),
            WordListGenerationError::WebPageNotFetched => "The web page could not be fetched".to_string(),
            WordListGenerationError::UnknownContentType => "Content-Type is not present or invalid".to_string(),
            WordListGenerationError::EmptyWordList => "No words in word list".to_string(),
        };
        write!(f, "{}", desc)
    }
}

#[derive(Debug)]
pub enum IndexGenerationError {
    NoFilesSpecified,
    NoValidFiles,
    DocumentErrors(Vec<DocumentError>),
}

impl Error for IndexGenerationError {}

impl fmt::Display for IndexGenerationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let desc: String = match self {
            IndexGenerationError::NoFilesSpecified => {
                "No files specified in config file".to_string()
            }
            IndexGenerationError::NoValidFiles => "No files could be indexed".to_string(),
            IndexGenerationError::DocumentErrors(errors) => format!(
                "{} {} found while indexing files. If you want to fail silently and still build an index, remove `break_on_file_error` from your config.\n{}",
                errors.len(),
                if errors.len() == 1 { "error" } else {"errors"},
                errors
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<String>>()
                    .join("\n")
            )
        };

        write!(f, "{}", desc)
    }
}
/**
 * Associates a `WordListGenerationError` with a `File`.
 */
#[derive(Debug, Clone)]
pub struct DocumentError {
    pub file: File,
    pub word_list_generation_error: WordListGenerationError,
}

impl fmt::Display for DocumentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "- {}\n  {}",
            self.word_list_generation_error.to_string().yellow(),
            self.file
        )
    }
}
