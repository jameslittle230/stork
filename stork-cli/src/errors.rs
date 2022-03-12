use std::{io, num::ParseIntError};
use stork_lib::{BuildError, ConfigReadError, IndexParseError, SearchError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StorkCommandLineError {
    #[error("Couldn't read the configuration file: {0}")]
    ConfigReadError(#[from] ConfigReadError),

    #[error("Couldn't read file `{0}`. Got error `{1}`")]
    FileReadError(String, io::Error),

    #[error("Couldn't create file `{0}`. Got error `{1}`")]
    FileCreateError(String, io::Error),

    #[error("Couldn't write to output stream. Got error `{0}`")]
    WriteError(io::Error),

    #[error("Stork doesn't support interactive stdin! Pipe in a stream instead.")]
    InteractiveStdinNotAllowed,

    #[error("{0}")]
    IndexParseError(#[from] IndexParseError),

    #[error("{0}")]
    BuildError(#[from] BuildError),

    #[error("Invalid port `{0}`. Got error `{1}`")]
    InvalidPort(String, ParseIntError),

    #[error("Couldn't start web server.")]
    ServerError,

    #[error("{0}")]
    SearchError(#[from] SearchError),

    #[error("Couldn't display search results as JSON. Got error `{0}`")]
    SearchResultJsonSerializationError(#[from] serde_json::Error),

    #[error("`{0}`")]
    InvalidCommandLineArguments(&'static str),
}
