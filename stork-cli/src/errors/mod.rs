use std::io;
use stork_lib as lib;
use thiserror::Error;

#[allow(dead_code)]
#[derive(Debug, Error)]
pub enum CommandLineError {
    #[error("Couldn't read the configuration file: {0}")]
    ConfigReadError(#[from] lib::config::ConfigReadError),

    #[error("Couldn't read file `{0}`. Got error `{1}`")]
    FileReadError(String, io::Error),

    #[error("Couldn't create file `{0}`. Got error `{1}`")]
    FileCreateError(String, io::Error),

    #[error("Couldn't write to output stream. Got error `{0}`")]
    WriteError(io::Error),

    #[error("Stork doesn't support interactive stdin! Pipe in a stream instead.")]
    InteractiveStdinNotAllowed,

    #[error("{0}")]
    IndexParseError(#[from] lib::parse_index::IndexParseError),

    #[error("{0}")]
    BuildError(#[from] lib::build_output::error::BuildError),

    #[error("Invalid port `{0}`. Got error `{1}`")]
    InvalidPort(String, clap::parser::MatchesError),

    #[error("Couldn't start web server.")]
    ServerError,

    #[error("Couldn't display search results as JSON. Got error `{0}`")]
    SearchResultJsonSerializationError(#[from] serde_json::Error),

    #[error("{0}")]
    InvalidCommandLineArguments(&'static str),

    #[error("{0}")]
    NotCompiledWithFeature(&'static str),
}