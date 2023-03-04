#![allow(clippy::module_name_repetitions)]

//! The object to define the content sources and configuration values for building a search index.
//!
//! There are two components to a search index configuration: the `InputConfig` and the `OutputConfig`.
//!

use bstr::ByteSlice;
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;
use ts_rs::TS;

mod file;
mod frontmatter;
mod html;
mod input;
mod local;
mod output;
mod srt;
mod stemming;

pub use self::frontmatter::FrontmatterConfig;
pub use self::html::HTMLConfig;
pub use file::{DataSource, File, Filetype};
pub use input::{InputConfig, TitleBoost};
pub use local::LocalConfig;
pub use output::OutputConfig;
pub use srt::{SRTConfig, SRTTimestampFormat};
pub use stemming::StemmingConfig;

pub mod errors;

#[derive(Serialize, Deserialize, Debug, SmartDefault, PartialEq, Eq, TS)]
#[ts(export)]
#[serde(default)]
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
            .and_then(|file| file.stemming.clone())
            .unwrap_or_else(|| self.input.stemming.clone());
    }

    pub(crate) fn get_srt_config_for_file(&self, file_index: usize) -> SRTConfig {
        return self
            .input
            .files
            .get(file_index)
            .and_then(|file| file.srt_config.clone())
            .unwrap_or_else(|| self.input.srt_config.clone());
    }

    pub(crate) fn get_frontmatter_config_for_file(&self, file_index: usize) -> FrontmatterConfig {
        self.input
            .files
            .get(file_index)
            .and_then(|file| file.frontmatter_config.clone())
            .unwrap_or_else(|| self.input.frontmatter_config.clone())
    }

    pub(crate) fn get_html_config_for_file(&self, file_index: usize) -> HTMLConfig {
        self.input
            .files
            .get(file_index)
            .and_then(|file| file.html_config.clone())
            .unwrap_or_else(|| self.input.html_config.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn empty_string_via_tryfrom_returns_error() {
        let contents = r#""#;
        let error = Config::try_from(contents).unwrap_err();
        assert_eq!(error, errors::ConfigReadError::EmptyString);
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
    fn bad_file_object_fails() {
        let contents = r#"
[input]
files = [{}]
    "#;
        let result = Config::try_from(contents).unwrap_err();
        let computed = result.to_string();
        let expected = "Cannot parse config as TOML. Stork recieved error: `missing field `title` for key `input.files` at line 3 column 10`";
        assert_eq!(computed, expected);
    }

    #[test]
    fn stem_config_no_override() {
        let config = Config {
            input: InputConfig {
                files: vec![File {
                    ..Default::default()
                }],
                stemming: StemmingConfig::Portuguese,
                ..Default::default()
            },
            ..Default::default()
        };

        assert_eq!(
            config.get_stem_config_for_file(0),
            StemmingConfig::Portuguese
        )
    }

    #[test]
    fn stem_config_with_override() {
        let config = Config {
            input: InputConfig {
                files: vec![File {
                    stemming: Some(StemmingConfig::Portuguese),
                    ..Default::default()
                }],
                stemming: StemmingConfig::Portuguese,
                ..Default::default()
            },
            ..Default::default()
        };

        assert_eq!(config.get_stem_config_for_file(0), StemmingConfig::Danish)
    }

    #[test]
    fn frontmatter_config_no_override() {
        let config = Config {
            input: InputConfig {
                files: vec![File {
                    ..Default::default()
                }],
                frontmatter_config: FrontmatterConfig::Omit,
                ..Default::default()
            },
            ..Default::default()
        };

        assert_eq!(
            config.get_frontmatter_config_for_file(0),
            FrontmatterConfig::Omit
        )
    }

    #[test]
    fn frontmatter_config_with_override() {
        let config = Config {
            input: InputConfig {
                files: vec![File {
                    frontmatter_config: Some(FrontmatterConfig::Parse),
                    ..Default::default()
                }],
                frontmatter_config: FrontmatterConfig::Omit,
                ..Default::default()
            },
            ..Default::default()
        };

        assert_eq!(
            config.get_frontmatter_config_for_file(0),
            FrontmatterConfig::Parse
        )
    }

    #[test]
    fn html_config_no_override() {
        let config = Config {
            input: InputConfig {
                files: vec![File {
                    ..Default::default()
                }],
                html_config: HTMLConfig {
                    save_nearest_id: true,
                    included_selectors: vec![".article".to_string()],
                    excluded_selectors: vec![".ignore".to_string()],
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        };

        assert_eq!(
            config.get_html_config_for_file(0),
            HTMLConfig {
                save_nearest_id: true,
                included_selectors: vec![".article".to_string()],
                excluded_selectors: vec![".ignore".to_string()],
                ..Default::default()
            }
        )
    }

    #[test]
    fn html_config_with_override() {
        let config = Config {
            input: InputConfig {
                files: vec![File {
                    html_config: Some(HTMLConfig {
                        save_nearest_id: false,
                        included_selectors: vec![".article-2".to_string()],
                        excluded_selectors: vec![".ignore-2".to_string()],
                        ..Default::default()
                    }),
                    ..Default::default()
                }],
                html_config: HTMLConfig {
                    save_nearest_id: true,
                    included_selectors: vec![".article".to_string()],
                    excluded_selectors: vec![".ignore".to_string()],
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        };

        assert_eq!(
            config.get_html_config_for_file(0),
            HTMLConfig {
                save_nearest_id: false,
                included_selectors: vec![".article-2".to_string()],
                excluded_selectors: vec![".ignore-2".to_string()],
                ..Default::default()
            }
        )
    }
}
