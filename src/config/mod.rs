use serde::{Deserialize, Serialize};
use std::fs;

mod file;
pub use file::DataSource;
pub use file::File;
pub use file::Filetype;

pub mod stemming_config;
pub use stemming_config::StemmingConfig;

pub mod frontmatter_config;
pub use frontmatter_config::FrontmatterConfig;

pub mod config_read_err;
use config_read_err::ConfigReadErr;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub enum TitleBoost {
    Minimal,
    Moderate,
    Large,
    Ridiculous,
}

impl Default for TitleBoost {
    fn default() -> Self {
        TitleBoost::Moderate
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields, default)]
pub struct Config {
    pub input: InputConfig,
    pub output: OutputConfig,
}

impl Config {
    pub fn from_file(path: std::path::PathBuf) -> Result<Config, ConfigReadErr> {
        let contents =
            fs::read_to_string(&path).map_err(|_| ConfigReadErr::UnreadableFile(path))?;
        toml::from_str(&contents).map_err(ConfigReadErr::UnparseableInput)
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            input: InputConfig::default(),
            output: OutputConfig::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields, default)]
#[allow(non_snake_case)]
pub struct InputConfig {
    #[serde(rename = "surrounding_word_count")]
    pub UNUSED_surrounding_word_count: Option<u8>,
    pub base_directory: String,
    pub url_prefix: String,
    pub title_boost: TitleBoost,
    pub stemming: StemmingConfig,

    #[serde(default)]
    pub html_selector: Option<String>,
    pub frontmatter_handling: FrontmatterConfig,
    pub files: Vec<File>,
    pub srt_config: SRTConfig,
    pub minimum_indexed_substring_length: u8,
    pub minimum_index_ideographic_substring_length: u8,
}

impl Default for InputConfig {
    fn default() -> Self {
        InputConfig {
            UNUSED_surrounding_word_count: Default::default(),
            base_directory: Default::default(),
            url_prefix: Default::default(),
            title_boost: Default::default(),
            stemming: Default::default(),
            html_selector: Default::default(),
            frontmatter_handling: Default::default(),
            files: Default::default(),
            srt_config: Default::default(),

            minimum_indexed_substring_length: 3,
            minimum_index_ideographic_substring_length: 1,
        }
    }
}

/**
 * Configuration settings that either:
 * - Alter how the index file is written to disk OR
 * - Gets "burned into" the index file and is passed through to the Javascript
 *   runtime, and thus can be overridden in JS.
 */
#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields, default)]
pub struct OutputConfig {
    pub filename: String,
    pub debug: bool,
    pub excerpt_buffer: u8,
    pub excerpts_per_result: u8,
    pub displayed_results_count: u8,
}

impl Default for OutputConfig {
    fn default() -> Self {
        OutputConfig {
            filename: "output.st".to_string(),
            debug: false,
            excerpt_buffer: 8,
            excerpts_per_result: 5,
            displayed_results_count: 10,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SRTConfig {
    pub timestamp_linking: bool,
    pub timestamp_template_string: String,
    pub timestamp_format: SRTTimestampFormat,
}

impl Default for SRTConfig {
    fn default() -> Self {
        SRTConfig {
            timestamp_linking: true,
            timestamp_template_string: "&t={ts}".to_string(),
            timestamp_format: SRTTimestampFormat::NumberOfSeconds,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum SRTTimestampFormat {
    NumberOfSeconds,
}

#[cfg(test)]
mod tests {
    use super::Config;
    use toml::de::Error;

    #[test]
    fn empty_file() -> Result<(), Error> {
        let contents = r#""#;
        toml::from_str(contents).map(|_c: Config| ())
    }

    #[test]
    fn simple_config() -> Result<(), Error> {
        let contents = r#"
[input]
base_directory = "test/federalist"
files = [
    {path = "federalist-1.txt", url = "https://www.congress.gov/resources/display/content/The+Federalist+Papers#TheFederalistPapers-1", title = "Introduction"},
    {path = "federalist-2.txt", url = "https://www.congress.gov/resources/display/content/The+Federalist+Papers#TheFederalistPapers-2", title = "Concerning Dangers from Foreign Force and Influence"},
    {path = "federalist-3.txt", url = "https://www.congress.gov/resources/display/content/The+Federalist+Papers#TheFederalistPapers-3", title = "Concerning Dangers from Foreign Force and Influence 2"},
]

[output]
filename = "test/federalist.st"
debug = false
    "#;
        toml::from_str(contents).map(|_c: Config| ())
    }

    #[test]
    fn surrounding_word_count_in_input() -> Result<(), Error> {
        let contents = r#"
[input]
base_directory = "test/federalist"
surrounding_word_count = 2
files = []

[output]
    "#;
        toml::from_str(contents).map(|_c: Config| ())
    }

    #[test]
    fn unknown_key_fails() {
        let contents = r#"
[bad_key]
    "#;
        let result: Result<Config, Error> = toml::from_str(contents);

        match result {
            Result::Ok(_r) => panic!("Config creation didn't fail with an unknown key"),
            Result::Err(_e) => (),
        }
    }

    #[test]
    fn empty_file_not_allowed() {
        let contents = r#"
[input]
files = [{}]
    "#;
        let result: Result<Config, Error> = toml::from_str(contents);

        match result {
            Result::Ok(_r) => panic!("Config creation didn't fail with an empty file object"),
            Result::Err(_e) => (),
        }
    }

    #[test]
    fn file_with_only_title_not_allowed() {
        let contents = r#"
[input]
files = [{title = "Derp"}]
    "#;
        let result: Result<Config, Error> = toml::from_str(contents);

        match result {
        Result::Ok(_r) => panic!("Config creation didn't fail with a file object that only had a title. File objects should have a title, url, and data source."),
        Result::Err(_e) => ()
    }
    }

    #[test]
    fn file_with_title_and_url_not_allowed() {
        let contents = r#"
[[input.files]]
title = "Derp"
url = "blorp"
    "#;
        let result: Result<Config, Error> = toml::from_str(contents);

        match result {
        Result::Ok(_r) => panic!("Config creation didn't fail with a file object that only had a title. File objects should have a title, url, and data source."),
        Result::Err(_e) => ()
    }
    }

    #[test]
    fn file_with_title_url_and_datasource_is_allowed() -> Result<(), Error> {
        let contents = r#"
[[input.files]]
title = "Derp"
url = "blorp"
contents = "According to all known laws of aviation, there is no way that a bee should be able to fly."
    "#;
        toml::from_str(contents).map(|_c: Config| ())
    }
}
