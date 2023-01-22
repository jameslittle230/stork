#![allow(clippy::module_name_repetitions)]

//! The object to define the content sources and configuration values for building a search index.
//!
//! There are two components to a search index configuration: the `InputConfig` and the `OutputConfig`.
//!

use bstr::ByteSlice;
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

mod file;
mod frontmatter;
mod html;
mod input;
mod local;
mod output;
mod srt;
mod stemming;

pub use self::frontmatter::FrontmatterConfig;
use self::html::HTMLConfig;
pub use file::{DataSource, File, Filetype};
pub use input::{InputConfig, TitleBoost};
pub use local::LocalConfig;
pub use output::OutputConfig;
pub use srt::{SRTConfig, SRTTimestampFormat};
pub use stemming::StemmingConfig;

pub mod errors;

#[derive(Serialize, Deserialize, Debug, SmartDefault, PartialEq, Eq)]
#[serde(deny_unknown_fields, default)]
pub struct Config {
    pub input: InputConfig,
    pub output: OutputConfig,

    #[serde(skip)]
    pub local: LocalConfig,
}

impl TryFrom<Bytes> for Config {
    type Error = errors::ConfigReadError;

    fn try_from(bytes: Bytes) -> Result<Self, Self::Error> {
        let string = bytes.to_str()?;
        Self::try_from(string)
    }
}

impl TryFrom<&str> for Config {
    type Error = errors::ConfigReadError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err(errors::ConfigReadError::EmptyString);
        }

        // TODO: Can I infer what filetype I'm seeing and pass that info into this function so I'm not literally trying to decode two things at once?
        let toml_output = toml::from_str::<Self>(value);
        let json_output = serde_json::from_str::<Self>(value);

        match (toml_output, json_output) {
            (Ok(toml_config), _) => Ok(toml_config),

            (Err(_), Ok(json_config)) => Ok(json_config),

            (Err(toml_error), Err(json_error)) => {
                // TODO: Use dtolnay's serde error path crate
                if let Some((mut toml_line, mut toml_col)) = toml_error.line_col() {
                    toml_line += 1;
                    toml_col += 1;
                    if toml_line > json_error.line()
                        || (toml_line == json_error.line() && toml_col > json_error.column())
                    {
                        Err(errors::ConfigReadError::UnparseableTomlInput(toml_error))
                    } else {
                        Err(errors::ConfigReadError::UnparseableJsonInput(json_error))
                    }
                } else {
                    Err(errors::ConfigReadError::UnparseableJsonInput(json_error))
                }
            }
        }
    }
}

impl Config {
    pub(crate) fn get_stem_config_for_file(&self, file_index: usize) -> StemmingConfig {
        return self
            .input
            .files
            .get(file_index)
            .map(|file| file.stemming.clone())
            .flatten()
            .unwrap_or_else(|| self.input.stemming.clone());
    }

    pub(crate) fn get_frontmatter_config_for_file(&self, file_index: usize) -> FrontmatterConfig {
        self.input
            .files
            .get(file_index)
            .map(|file| file.frontmatter_config.clone())
            .flatten()
            .unwrap_or_else(|| self.input.frontmatter_config.clone())
    }

    pub(crate) fn get_html_config_for_file(&self, file_index: usize) -> HTMLConfig {
        self.input
            .files
            .get(file_index)
            .map(|file| file.html_config.clone())
            .flatten()
            .unwrap_or_else(|| self.input.html_config.clone())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::{html::HTMLConfig, *};
    use pretty_assertions::assert_eq;

    #[test]
    fn empty_string_via_tryfrom_returns_error() {
        let contents = r#""#;
        let error = Config::try_from(contents).unwrap_err();
        assert_eq!(error, errors::ConfigReadError::EmptyString);
    }

    fn get_default_config() -> Config {
        Config {
            input: InputConfig {
                base_directory: "test/federalist".into(),
                url_prefix: "".into(),
                title_boost: TitleBoost::Moderate,
                stemming: StemmingConfig::Language(
                    rust_stemmers::Algorithm::English,
                ),
                frontmatter_config: FrontmatterConfig::Omit,
                files: vec![
                    File {title:"Introduction".into(),url:"https://www.congress.gov/resources/display/content/The+Federalist+Papers#TheFederalistPapers-1".into(),explicit_source:Some(DataSource::FilePath("federalist-1.txt".into(),),),filetype:None,fields:HashMap::new(), stemming: None, html_config: None, frontmatter_config: None, srt_config: None },
                    File {title:"Concerning Dangers from Foreign Force and Influence".into(),url:"https://www.congress.gov/resources/display/content/The+Federalist+Papers#TheFederalistPapers-2".into(),explicit_source:Some(DataSource::FilePath("federalist-2.txt".into(),),),filetype:None,fields:HashMap::new(), stemming: None, html_config: None, frontmatter_config: None, srt_config: None },
                    File {title:"Concerning Dangers from Foreign Force and Influence 2".into(),url:"https://www.congress.gov/resources/display/content/The+Federalist+Papers#TheFederalistPapers-3".into(),explicit_source:Some(DataSource::FilePath("federalist-3.txt".into(),),),filetype:None,fields:HashMap::new(), stemming: None, html_config: None, frontmatter_config: None, srt_config: None },
                ],
                srt_config: SRTConfig {
                    timestamp_linking: true,
                    timestamp_template_string: "&t={ts}".into(),
                    timestamp_format: SRTTimestampFormat::NumberOfSeconds,
                },
                html_config: HTMLConfig {
                    save_nearest_id: true,
                    included_selectors: vec!["main".to_string()],
                    excluded_selectors: vec![],
                },
            },
            output: OutputConfig {
                chunk_size_kb: 0,
                excerpt_buffer: 8,
                excerpts_per_result: 5,
                displayed_results_count: 10,
                minimum_query_length: 3,
                break_on_file_error: true,
            },
            local: LocalConfig { debug_output: false },
        }
    }

    // This test also makes sure that our default values don't change
    // without being accounted for in tests.
    #[test]
    fn simple_toml_config_is_parseable() {
        let contents = r#"
[input]
base_directory = "test/federalist"
files = [
    {path = "federalist-1.txt", url = "https://www.congress.gov/resources/display/content/The+Federalist+Papers#TheFederalistPapers-1", title = "Introduction"},
    {path = "federalist-2.txt", url = "https://www.congress.gov/resources/display/content/The+Federalist+Papers#TheFederalistPapers-2", title = "Concerning Dangers from Foreign Force and Influence"},
    {path = "federalist-3.txt", url = "https://www.congress.gov/resources/display/content/The+Federalist+Papers#TheFederalistPapers-3", title = "Concerning Dangers from Foreign Force and Influence 2"},
]
    "#;

        let computed = Config::try_from(contents).unwrap();
        let expected = get_default_config();

        assert_eq!(computed, expected);
    }

    #[test]
    fn simple_json_config_is_parseable() {
        let contents = r#"
        {
            "input": {
                "base_directory": "test/federalist",
                "files": [
                    {
                        "path": "federalist-1.txt",
                        "url": "https://www.congress.gov/resources/display/content/The+Federalist+Papers#TheFederalistPapers-1",
                        "title": "Introduction"
                    },
                    {
                        "path": "federalist-2.txt",
                        "url": "https://www.congress.gov/resources/display/content/The+Federalist+Papers#TheFederalistPapers-2",
                        "title": "Concerning Dangers from Foreign Force and Influence"
                    },
                    {
                        "path": "federalist-3.txt",
                        "url": "https://www.congress.gov/resources/display/content/The+Federalist+Papers#TheFederalistPapers-3",
                        "title": "Concerning Dangers from Foreign Force and Influence 2"
                    }
                ]
            }
        }
    "#;

        let computed = Config::try_from(contents).unwrap();
        let expected = get_default_config();

        assert_eq!(computed, expected);
    }

    #[test]
    fn bad_toml_syntax_fails_with_toml_error() {
        let contents = r#"[input] {}"#;
        let error = Config::try_from(contents).unwrap_err();
        let computed = error.to_string();
        let expected = "Cannot parse config as TOML. Stork recieved error: `expected newline, found a left brace at line 1 column 9`";
        assert_eq!(computed, expected);
    }
    #[test]
    fn bad_json_syntax_fails_with_json_error() {
        let contents = r#"{"input", ]}"#;
        let error = Config::try_from(contents).unwrap_err();
        let computed = error.to_string();
        let expected =
            "Cannot parse config as JSON. Stork recieved error: `expected `:` at line 1 column 9`";
        assert_eq!(computed, expected);
    }

    #[test]
    fn empty_file_array_fails() {
        let contents = r#"
[input]
files = [{}]
    "#;
        let result: toml::de::Error = toml::from_str::<Config>(contents).unwrap_err();
        let computed = result.to_string();
        let expected = "missing field `title` for key `input.files` at line 3 column 10"; // TODO: Can this be nicer?
        assert_eq!(computed, expected);
    }
}
