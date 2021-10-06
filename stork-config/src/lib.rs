use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;
use std::path::PathBuf;

mod stemming;
pub use stemming::StemmingConfig;

mod frontmatter;
pub use frontmatter::FrontmatterConfig;

mod file;
pub use file::DataSource;
pub use file::File;
pub use file::Filetype;

mod srt;
pub use srt::SRTConfig;
pub use srt::SRTTimestampFormat;

mod errors;
pub use errors::ConfigFromFileError;

#[derive(Serialize, Deserialize, Debug, SmartDefault, PartialEq)]
#[serde(deny_unknown_fields, default)]
pub struct Config {
    pub input: InputConfig,
    pub output: OutputConfig,
}

#[derive(Serialize, Deserialize, Debug, Clone, SmartDefault, PartialEq)]
#[serde(deny_unknown_fields, default)]
#[allow(non_snake_case)]
pub struct InputConfig {
    #[serde(rename = "surrounding_word_count")]
    pub UNUSED_surrounding_word_count: Option<u8>,
    pub base_directory: String,
    pub url_prefix: String,
    pub title_boost: TitleBoost,
    pub stemming: StemmingConfig,
    pub html_selector: Option<String>,

    #[default(None)]
    pub exclude_html_selector: Option<String>,

    pub frontmatter_handling: FrontmatterConfig,
    pub files: Vec<File>,

    #[default = false]
    pub break_on_file_error: bool,
    pub srt_config: SRTConfig,

    #[default = 3]
    pub minimum_indexed_substring_length: u8,

    #[default = 1]
    pub minimum_index_ideographic_substring_length: u8,
}

#[derive(Serialize, Deserialize, Debug, SmartDefault, PartialEq)]
#[serde(deny_unknown_fields, default)]
#[allow(non_snake_case)]
pub struct OutputConfig {
    #[serde(rename = "filename")]
    #[default(None)]
    pub UNUSED_filename: Option<String>,

    #[default = false]
    pub debug: bool,

    #[default = 8]
    pub excerpt_buffer: u8,

    #[default = 5]
    pub excerpts_per_result: u8,

    #[default = 10]
    pub displayed_results_count: u8,
}

#[derive(Serialize, Deserialize, Clone, Debug, SmartDefault, PartialEq)]
#[serde(deny_unknown_fields)]
pub enum TitleBoost {
    Minimal,
    #[default]
    Moderate,
    Large,
    Ridiculous,
}

impl TryFrom<&str> for Config {
    type Error = ConfigFromFileError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err(ConfigFromFileError::EmptyString);
        }

        toml::from_str(value).map_err(ConfigFromFileError::UnparseableInput)
    }
}

impl TryFrom<&PathBuf> for Config {
    type Error = ConfigFromFileError;

    fn try_from(value: &PathBuf) -> Result<Self, Self::Error> {
        let contents = std::fs::read_to_string(value)
            .map_err(|_| ConfigFromFileError::UnreadableFile(value.into()))?;
        Config::try_from(contents.as_str())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn empty_string_returns_default_config() {
        let contents = r#""#;
        let config: Config = toml::from_str(contents).unwrap();
        assert_eq!(config, Config::default())
    }

    #[test]
    fn empty_string_via_tryfrom_returns_error() {
        let contents = r#""#;
        let error = Config::try_from(contents).unwrap_err();
        assert_eq!(error, ConfigFromFileError::EmptyString)
    }

    /// This test also makes sure that our default values don't change without being accounted for in tests.
    #[test]
    fn simple_config_is_parseable() {
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
debug = true
    "#;

        let computed: Config = toml::from_str(contents).unwrap();

        let expected = Config {
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
                UNUSED_filename: Some(
                    "test/federalist.st".to_string(),
                ),
                debug: true,
                excerpt_buffer: 8,
                excerpts_per_result: 5,
                displayed_results_count: 10,
            },
        };

        assert_eq!(computed, expected)
    }

    #[test]
    fn unknown_top_level_key_fails_with_toml_error() {
        let contents = r#"[bad_key]"#;
        let result: toml::de::Error = toml::from_str::<Config>(contents).unwrap_err();
        let computed = result.to_string();
        let expected = "unknown field `bad_key`, expected `input` or `output` at line 1 column 1";
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
