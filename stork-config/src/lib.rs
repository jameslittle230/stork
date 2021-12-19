use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

mod input;
pub use input::{InputConfig, TitleBoost};

mod output;
pub use output::OutputConfig;

mod stemming;
pub use stemming::StemmingConfig;

mod frontmatter;
pub use frontmatter::FrontmatterConfig;

mod file;
pub use file::{DataSource, File, Filetype};

mod srt;
pub use srt::{SRTConfig, SRTTimestampFormat};

mod errors;
pub use errors::ConfigReadError;

#[derive(Serialize, Deserialize, Debug, SmartDefault, PartialEq)]
#[serde(deny_unknown_fields, default)]
pub struct Config {
    pub input: InputConfig,
    pub output: OutputConfig,
}

impl TryFrom<&str> for Config {
    type Error = ConfigReadError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err(ConfigReadError::EmptyString);
        }

        let toml_output = toml::from_str::<Self>(value);
        let json_output = serde_json::from_str::<Self>(value);

        match (toml_output, json_output) {
            (Ok(toml_config), _) => Ok(toml_config),

            (Err(_), Ok(json_config)) => Ok(json_config),

            (Err(toml_error), Err(json_error)) => {
                if let Some((mut toml_line, mut toml_col)) = toml_error.line_col() {
                    toml_line += 1;
                    toml_col += 1;
                    dbg!(toml_line, toml_col, json_error.line(), json_error.column());
                    if toml_line > json_error.line()
                        || (toml_line == json_error.line() && toml_col > json_error.column())
                    {
                        Err(ConfigReadError::UnparseableTomlInput(toml_error))
                    } else {
                        Err(ConfigReadError::UnparseableJsonInput(json_error))
                    }
                } else {
                    Err(ConfigReadError::UnparseableJsonInput(json_error))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn empty_string_via_tryfrom_returns_error() {
        let contents = r#""#;
        let error = Config::try_from(contents).unwrap_err();
        assert_eq!(error, ConfigReadError::EmptyString)
    }

    fn get_default_config() -> Config {
        Config {
            input: InputConfig {
                UNUSED_surrounding_word_count: None,
                base_directory: "test/federalist".into(),
                url_prefix: "".into(),
                title_boost: TitleBoost::Moderate,
                stemming: StemmingConfig::Language(
                    rust_stemmers::Algorithm::English,
                ),
                html_selector: None,
                exclude_html_selector: None,
                frontmatter_handling: FrontmatterConfig::Omit,
                files: vec![
                    File {
                        title: "Introduction".into(),
                        url: "https://www.congress.gov/resources/display/content/The+Federalist+Papers#TheFederalistPapers-1".into(),
                        explicit_source: Some(
                            DataSource::FilePath(
                                "federalist-1.txt".into(),
                            ),
                        ),
                        id: None,
                        stemming_override: None,
                        html_selector_override: None,
                        exclude_html_selector_override: None,
                        frontmatter_handling_override: None,
                        filetype: None,
                        fields: HashMap::new(),
                    },
                    File {
                        title: "Concerning Dangers from Foreign Force and Influence".into(),
                        url: "https://www.congress.gov/resources/display/content/The+Federalist+Papers#TheFederalistPapers-2".into(),
                        explicit_source: Some(
                            DataSource::FilePath(
                                "federalist-2.txt".into(),
                            ),
                        ),
                        id: None,
                        stemming_override: None,
                        html_selector_override: None,
                        exclude_html_selector_override: None,
                        frontmatter_handling_override: None,
                        filetype: None,
                        fields: HashMap::new(),
                    },
                    File {
                        title: "Concerning Dangers from Foreign Force and Influence 2".into(),
                        url: "https://www.congress.gov/resources/display/content/The+Federalist+Papers#TheFederalistPapers-3".into(),
                        explicit_source: Some(
                            DataSource::FilePath(
                                "federalist-3.txt".into(),
                            ),
                        ),
                        id: None,
                        stemming_override: None,
                        html_selector_override: None,
                        exclude_html_selector_override: None,
                        frontmatter_handling_override: None,
                        filetype: None,
                        fields: HashMap::new(),
                    },
                ],
                break_on_file_error: false,
                srt_config: SRTConfig {
                    timestamp_linking: true,
                    timestamp_template_string: "&t={ts}".into(),
                    timestamp_format: SRTTimestampFormat::NumberOfSeconds,
                },
                minimum_indexed_substring_length: 3,
                minimum_index_ideographic_substring_length: 1,
            },
            output: OutputConfig {
                UNUSED_filename: None,
                debug: true,
                save_nearest_html_id: false,
                excerpt_buffer: 8,
                excerpts_per_result: 5,
                displayed_results_count: 10,
            },
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

[output]
debug = true
    "#;

        let computed = Config::try_from(contents).unwrap();
        let expected = get_default_config();

        assert_eq!(computed, expected)
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
            },
            "output": {
                "debug": true
            }
        }
    "#;

        let computed = Config::try_from(contents).unwrap();
        let expected = get_default_config();

        assert_eq!(computed, expected)
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
