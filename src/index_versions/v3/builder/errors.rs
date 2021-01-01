use std::{error::Error, fmt};
use crate::config::File;
use super::word_list_generators::WordListGenerationError;

#[derive(Debug)]
pub enum IndexGenerationError {
    NoFilesSpecified,
}

impl Error for IndexGenerationError {}

impl fmt::Display for IndexGenerationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let desc: String = match self {
            IndexGenerationError::NoFilesSpecified => {
                "No files specified in config file".to_string()
            }
        };

        write!(f, "{}", desc)
    }
}

pub struct DocumentError {
    pub file: File,
    pub word_list_generation_error: WordListGenerationError,
}

impl fmt::Display for DocumentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Error indexing `{}`: {}",
            self.file, self.word_list_generation_error
        )
    }
}
