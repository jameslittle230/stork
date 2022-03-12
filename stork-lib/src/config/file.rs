use super::{FrontmatterConfig, StemmingConfig};
use core::fmt;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;
use std::collections::HashMap;

type Fields = HashMap<String, String>;

#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct File {
    pub title: String,
    pub url: String,

    /// Implicit source will take from the destination URL
    #[serde(flatten)]
    pub explicit_source: Option<DataSource>,

    pub id: Option<String>,

    #[serde(default)]
    pub stemming_override: Option<StemmingConfig>,

    #[serde(default)]
    pub html_selector_override: Option<String>,

    #[serde(default)]
    pub exclude_html_selector_override: Option<String>,

    #[serde(default)]
    pub frontmatter_handling_override: Option<FrontmatterConfig>,

    #[serde(default)]
    pub filetype: Option<Filetype>,

    #[serde(flatten, default)]
    pub fields: Fields,
}

impl File {
    pub fn source(&self) -> DataSource {
        match &self.explicit_source {
            Some(source) => source.clone(),
            None => DataSource::URL(self.url.clone()),
        }
    }
}

impl fmt::Display for File {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match &self.source() {
                DataSource::FilePath(path) => path,
                DataSource::URL(url) => url,

                // if there's no string representation of where to find the file,
                // just use the title
                DataSource::Contents(_contents) => &self.title,
            }
        )
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, SmartDefault)]
pub enum DataSource {
    #[serde(rename = "contents")]
    #[default]
    Contents(String),

    #[serde(rename = "src_url")]
    URL(String),

    #[serde(rename = "path")]
    FilePath(String),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
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
        assert_eq!(file.explicit_source, None);
        assert_eq!(file.source(), DataSource::URL("blorp".into()));
        assert_eq!(file.url, "blorp");
    }

    #[test]
    fn json_file_with_title_and_url_assumes_url_is_source() {
        let json = r#"{"title": "Derp", "url": "blorp"}"#;
        let file: File = serde_json::from_str(json).unwrap();
        assert_eq!(file.explicit_source, None);
        assert_eq!(file.source(), DataSource::URL("blorp".into()));
        assert_eq!(file.url, "blorp");
    }

    #[test]
    fn file_with_explicit_url_source() {
        let toml = r#"title = "Derp"
        url = "blorp"
        src_url = "google.com""#;
        let file: File = toml::from_str(toml).unwrap();
        assert_eq!(
            file.explicit_source,
            Some(DataSource::URL("google.com".into()))
        );
        assert_eq!(file.source(), DataSource::URL("google.com".into()));
        assert_eq!(file.url, "blorp");
    }
    #[test]
    fn json_file_with_explicit_url_source() {
        let json = r#"{"title": "Derp", "url": "blorp", "src_url": "google.com"}"#;
        let file: File = serde_json::from_str(json).unwrap();
        assert_eq!(
            file.explicit_source,
            Some(DataSource::URL("google.com".into()))
        );
        assert_eq!(file.source(), DataSource::URL("google.com".into()));
        assert_eq!(file.url, "blorp");
    }

    #[test]
    fn file_with_only_src_url_fails() {
        let toml = r#"title = "Derp"
        src_url = "google.com""#;
        let error: Error = toml::from_str::<File>(toml).unwrap_err();
        let computed = error.to_string();
        let expected = "missing field `url` at line 1 column 1";
        assert_eq!(computed, expected);
    }

    #[test]
    fn json_file_with_only_src_url_fails() {
        let json = r#"{"title": "Derp", "src_url": "google.com"}"#;
        let error: JsonError = serde_json::from_str::<File>(json).unwrap_err();
        let computed = error.to_string();
        let expected = "missing field `url` at line 1 column 42";
        assert_eq!(computed, expected);
    }

    #[test]
    fn file_with_multiple_sources_fails() {
        let toml = r#"title = "Derp"
        url = "apple.com"
        src_url = "google.com"
        contents = "According to all known laws of aviation...""#;
        let error: Error = toml::from_str::<File>(toml).unwrap_err();
        let computed = error.to_string();
        let expected = "unknown field `contents` at line 1 column 1";
        assert_eq!(computed, expected);
    }

    #[test]
    fn json_file_with_multiple_sources_fails() {
        let json = r#"{"title": "Derp", "url": "apple.com", "src_url": "google.com", "contents": "According to all known laws of aviation..."}"#;
        let error: JsonError = serde_json::from_str::<File>(json).unwrap_err();
        let computed = error.to_string();
        let expected = "unknown field `contents` at line 1 column 120";
        assert_eq!(computed, expected);
    }

    #[test]
    fn file_with_multiple_sources_fails_contents_first() {
        let toml = r#"title = "Derp"
        url = "apple.com"
        contents = "According to all known laws of aviation..."
        src_url = "google.com""#;
        let error: Error = toml::from_str::<File>(toml).unwrap_err();
        let computed = error.to_string();
        let expected = "unknown field `src_url` at line 1 column 1";
        assert_eq!(computed, expected);
    }

    #[test]
    fn json_file_with_multiple_sources_fails_contents_first() {
        let json = r#"{"title": "Derp", "url": "apple.com",
        "contents": "According to all known laws of aviation...",
        "src_url": "google.com"}"#;
        let error: JsonError = serde_json::from_str::<File>(json).unwrap_err();
        let computed = error.to_string();
        let expected = "unknown field `src_url` at line 3 column 32";
        assert_eq!(computed, expected);
    }
}
