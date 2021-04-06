use std::{fmt::Display, io, num::ParseIntError};

use stork::{
    config::config_read_err::ConfigReadErr, LatestVersion::builder::errors::IndexGenerationError,
};

pub enum StorkCommandLineError {
    FileReadError(String, io::Error),
    FileCreateError(String, io::Error),
    WriteError(io::Error),
    InteractiveStdinNotAllowed,
    ConfigReadError(ConfigReadErr),
    IndexReadError,
    IndexGenerationError(IndexGenerationError),
    InvalidPort(String, ParseIntError),
    ServerError,
    SearchError,
    InvalidCommandLineArguments(String),
}

impl Display for StorkCommandLineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let data = match self {
            StorkCommandLineError::FileReadError(path, e) => {
                format!("Couldn't read file `{}`: {}", path, e)
            }
            StorkCommandLineError::FileCreateError(path, e) => {
                format!("Couldn't create file `{}`: {}", path, e)
            }
            StorkCommandLineError::WriteError(e) => {
                format!("Couldn't write to output stream: {}", e)
            }
            StorkCommandLineError::InteractiveStdinNotAllowed => {
                "Stork doesn't support interactive stdin! Pipe in a stream instead.".to_string()
            }
            StorkCommandLineError::ConfigReadError(e) => format!("{}", e),
            StorkCommandLineError::IndexReadError => "Couldn't read index".to_string(),
            StorkCommandLineError::IndexGenerationError(e) => format!("{}", e),
            StorkCommandLineError::InvalidPort(string, e) => {
                format!("Invalid port `{}`: {}", string, e)
            }
            StorkCommandLineError::ServerError => "Couldn't start web server".to_string(),
            StorkCommandLineError::SearchError => "Couldn't search".to_string(),
            StorkCommandLineError::InvalidCommandLineArguments(string) => {
                format!("Invalid command line arguments passed. {}", string)
            }
        };

        f.write_str(data.as_str())
    }
}
