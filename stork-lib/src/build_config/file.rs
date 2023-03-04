#![allow(clippy::upper_case_acronyms)]

use super::{html::HTMLConfig, FrontmatterConfig, SRTConfig, StemmingConfig};
use core::fmt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use ts_rs::TS;

type Fields = HashMap<String, String>;

#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq, Eq, TS)]
#[ts(export)]
pub struct File {
    pub title: String,
    pub url: String,

    #[serde(default)]
    pub fields: Fields,

    #[serde(default)]
    pub contents: Option<String>,

    #[serde(default)]
    pub src_url: Option<String>,

    #[serde(default)]
    pub path: Option<String>,

    #[serde(default)]
    pub stemming: Option<StemmingConfig>,

    #[serde(default)]
    pub html_config: Option<HTMLConfig>,

    #[serde(default)]
    pub frontmatter_config: Option<FrontmatterConfig>,

    #[serde(default)]
    pub srt_config: Option<SRTConfig>,

    #[serde(default)]
    pub filetype: Option<Filetype>,
}

impl File {
    pub fn source(&self) -> Result<DataSource, String> {
        match (&self.contents, &self.src_url, &self.path) {
            (Some(contents), None, None) => Ok(DataSource::Contents(contents.clone())),
            (None, Some(src_url), None) => Ok(DataSource::URL(src_url.clone())),
            (None, None, Some(path)) => Ok(DataSource::FilePath(path.clone())),
            (None, None, None) => Ok(DataSource::URL(self.url.clone())),
            _ => Err("Multiple content sources specified for file".to_string()),
        }
    }
}

impl fmt::Display for File {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match &self.source().unwrap() {
                DataSource::FilePath(path) => path,
                DataSource::URL(url) => url,

                // if there's no string representation of where to find the file,
                // just use the title
                DataSource::Contents(_contents) => &self.title,
            }
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DataSource {
    Contents(String),
    URL(String),
    FilePath(String),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, TS)]
#[ts(export)]
pub enum Filetype {
    PlainText,
    SRTSubtitle,
    HTML,
    Markdown,
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use serde_json::Error as JsonError;
    use toml::de::Error;

    #[test]
    fn file_with_only_title_fails() {
        let toml = r#"title = "Derp""#;
        let error: Error = toml::from_str::<File>(toml).unwrap_err();
        let computed = error.to_string();
        let expected = "missing field `url` at line 1 column 1";
        assert_eq!(computed, expected);
    }

    #[test]
    fn json_file_with_only_title_fails() {
        let json = r#"{"title": "Derp"}"#;
        let error: JsonError = serde_json::from_str::<File>(json).unwrap_err();
        let computed = error.to_string();
        let expected = "missing field `url` at line 1 column 17";
        assert_eq!(computed, expected);
    }

    #[test]
    fn file_with_title_and_url_assumes_url_is_source() {
        let toml = r#"title = "Derp"
        url = "blorp""#;
        let file: File = toml::from_str(toml).unwrap();
        assert_eq!(file.source(), Ok(DataSource::URL("blorp".into())));
        assert_eq!(file.url, "blorp");
    }

    #[test]
    fn json_file_with_title_and_url_assumes_url_is_source() {
        let json = r#"{"title": "Derp", "url": "blorp"}"#;
        let file: File = serde_json::from_str(json).unwrap();
        assert_eq!(file.source(), Ok(DataSource::URL("blorp".into())));
        assert_eq!(file.url, "blorp");
    }

    #[test]
    fn file_with_explicit_url_source() {
        let toml = r#"title = "Derp"
        url = "blorp"
        src_url = "google.com""#;
        let file: File = toml::from_str(toml).unwrap();
        assert_eq!(file.source(), Ok(DataSource::URL("google.com".into())));
        assert_eq!(file.url, "blorp");
    }
    #[test]
    fn json_file_with_explicit_url_source() {
        let json = r#"{"title": "Derp", "url": "blorp", "src_url": "google.com"}"#;
        let file: File = serde_json::from_str(json).unwrap();

        assert_eq!(file.source(), Ok(DataSource::URL("google.com".into())));
        assert_eq!(file.url, "blorp");
    }

    #[test]
    fn file_with_only_src_url_fails() {
        let toml = r#"title = "Derp"
        src_url = "google.com""#;
        let computed = toml::from_str::<File>(toml).unwrap_err().to_string();
        let expected = "missing field `url` at line 1 column 1";
        assert_eq!(computed, expected);
    }

    #[test]
    fn json_file_with_only_src_url_fails() {
        let json = r#"{"title": "Derp", "src_url": "google.com"}"#;
        let computed = serde_json::from_str::<File>(json).unwrap_err().to_string();
        let expected = "missing field `url` at line 1 column 42";
        assert_eq!(computed, expected);
    }

    #[test]
    fn file_with_multiple_sources_fails() {
        let toml = r#"title = "Derp"
        url = "apple.com"
        src_url = "google.com"
        contents = "According to all known laws of aviation...""#;
        let file: File = toml::from_str(toml).unwrap();
        let computed = file.source().unwrap_err();
        let expected = "Multiple content sources specified for file";
        assert_eq!(computed, expected);
    }

    #[test]
    fn json_file_with_multiple_sources_fails() {
        let json = r#"{"title": "Derp", "url": "apple.com", "src_url": "google.com", "contents": "According to all known laws of aviation..."}"#;
        let file: File = serde_json::from_str(json).unwrap();
        let computed = file.source().unwrap_err();
        let expected = "Multiple content sources specified for file";
        assert_eq!(computed, expected);
    }

    #[test]
    fn file_with_multiple_sources_fails_contents_first() {
        let toml = r#"title = "Derp"
        url = "apple.com"
        contents = "According to all known laws of aviation..."
        src_url = "google.com""#;
        let file: File = toml::from_str(toml).unwrap();
        let computed = file.source().unwrap_err();
        let expected = "Multiple content sources specified for file";
        assert_eq!(computed, expected);
    }

    #[test]
    fn json_file_with_multiple_sources_fails_contents_first() {
        let json = r#"{"title": "Derp", "url": "apple.com",
        "contents": "According to all known laws of aviation...",
        "src_url": "google.com"}"#;
        let file: File = serde_json::from_str(json).unwrap();
        let computed = file.source().unwrap_err();
        let expected = "Multiple content sources specified for file";
        assert_eq!(computed, expected);
    }
}
