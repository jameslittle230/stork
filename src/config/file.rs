use super::{FrontmatterConfig, StemmingConfig};
use core::fmt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

type Fields = HashMap<String, String>;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct File {
    pub title: String,
    pub url: String,
    #[serde(flatten)]
    pub source: DataSource,

    pub id: Option<String>,
    #[serde(default)]
    pub stemming_override: Option<StemmingConfig>,

    #[serde(default)]
    pub html_selector_override: Option<String>,

    #[serde(default)]
    pub frontmatter_handling_override: Option<FrontmatterConfig>,

    #[serde(default)]
    pub filetype: Option<Filetype>,

    #[serde(flatten, default)]
    pub fields: Fields,
}

impl fmt::Display for File {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match &self.source {
                DataSource::FilePath(path) => path,
                DataSource::Contents(_contents) => &self.title,
                DataSource::URL(url) => url,
            }
        )
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum DataSource {
    #[serde(rename = "contents")]
    Contents(String),

    #[serde(rename = "src_url")]
    URL(String),

    #[serde(rename = "path")]
    FilePath(String),
}

impl Default for DataSource {
    fn default() -> Self {
        DataSource::Contents(String::default())
    }
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
    // use super::*;
    #[test]
    #[ignore = "computed_filetype is no more"]
    fn compute_from_explicit_filetype() {
        // assert_eq!(
        //     File {
        //         filetype: Some(Filetype::PlainText),
        //         ..Default::default()
        //     }
        //     .computed_filetype()
        //     .unwrap(),
        //     Filetype::PlainText
        // )
    }

    #[test]
    #[ignore = "computed_filetype is no more"]
    fn compute_from_implicit_filetype_plaintext() {
        // assert_eq!(
        //     File {
        //         source: DataSource::FilePath("blah.txt".to_string()),
        //         ..Default::default()
        //     }
        //     .computed_filetype()
        //     .unwrap(),
        //     Filetype::PlainText
        // )
    }

    #[test]
    #[ignore = "computed_filetype is no more"]
    fn compute_from_implicit_filetype_html() {
        // assert_eq!(
        //     File {
        //         source: DataSource::FilePath("blah.html".to_string()),
        //         ..Default::default()
        //     }
        //     .computed_filetype()
        //     .unwrap(),
        //     Filetype::HTML
        // )
    }

    #[test]
    #[ignore = "computed_filetype is no more"]
    fn compute_from_implicit_filetype_srt() {
        // assert_eq!(
        //     File {
        //         source: DataSource::FilePath("blah.srt".to_string()),
        //         ..Default::default()
        //     }
        //     .computed_filetype()
        //     .unwrap(),
        //     Filetype::SRTSubtitle
        // )
    }

    #[test]
    #[ignore = "computed_filetype is no more"]
    fn compute_from_implicit_filetype_html_allcaps() {
        // assert_eq!(
        //     File {
        //         source: DataSource::FilePath("MYFILE.HTM".to_string()),
        //         ..Default::default()
        //     }
        //     .computed_filetype()
        //     .unwrap(),
        //     Filetype::HTML
        // )
    }

    #[test]
    #[ignore = "computed_filetype is no more"]
    fn compute_from_implicit_filetype_error() {
        // assert!(File {
        //     source: DataSource::FilePath("myfile.derp".to_string()),
        //     ..Default::default()
        // }
        // .computed_filetype()
        // .is_none())
    }

    #[test]
    #[ignore = "computed_filetype is no more"]
    fn compute_from_no_filetype_error() {
        // assert!(File {
        //     source: DataSource::Contents("A long time ago...".to_string()),
        //     ..Default::default()
        // }
        // .computed_filetype()
        // .is_none())
    }
}
