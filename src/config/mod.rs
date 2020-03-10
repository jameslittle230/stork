use crate::Fields;
use serde::Deserialize;
use std::fs;

#[derive(Deserialize)]
pub struct Config {
    pub input: InputConfig,
    pub output: OutputConfig,
}

impl Config {
    pub fn from_file(path: std::path::PathBuf) -> Config {
        let contents = fs::read_to_string(&path).unwrap_or_else(|_e| {
            panic!(
                "Something went wrong reading the file {}",
                &path.to_str().unwrap()
            )
        });

        toml::from_str(&contents).expect("Config file does not contain proper TOML syntax.")
    }
}

#[derive(Deserialize)]
pub struct InputConfig {
    pub surrounding_word_count: Option<u8>,
    pub base_directory: String,
    pub files: Option<Vec<InputFileListing>>,
}

#[derive(Deserialize)]
pub struct InputFileListing {
    pub path: String,
    pub url: String,
    pub title: String,
    pub fields: Fields,
}

#[derive(Deserialize)]
pub struct OutputConfig {
    pub filename: String,
    pub debug: Option<bool>,
}
