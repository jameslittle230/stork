use colored::Colorize;

use super::word_list_generators::WordListGenerationError;
use crate::config::File;
use std::{error::Error, fmt};

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
                    .map(|e| e.to_string())
                    .collect::<Vec<String>>()
                    .join("\n")
            )
            .to_string(),
        };

        write!(f, "{}", desc)
    }
}
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
