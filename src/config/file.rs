use super::{FrontmatterConfig, StemmingConfig};
use core::fmt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

type Fields = HashMap<String, String>;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
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
