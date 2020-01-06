use crate::models::StorkEntry;
use serde::Deserialize;
#[derive(Deserialize)]
pub struct Config {
    pub input: ConfigInput,
    pub output: ConfigOutput,
}

#[derive(Deserialize)]
pub struct ConfigInput {
    pub surrounding_word_count: Option<u8>,
    pub base_directory: String,
    pub files: Vec<StorkEntry>,
}

#[derive(Deserialize)]
pub struct ConfigOutput {
    pub filename: String,
    pub debug: bool,
}
