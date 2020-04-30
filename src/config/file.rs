use super::StemmingConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

type Fields = HashMap<String, String>;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct File {
    pub title: String,
    pub url: String,
    #[serde(flatten)]
    pub source: DataSource,

    pub id: Option<String>,
    #[serde(default)]
    pub stemming_override: Option<StemmingConfig>,

    #[serde(default)]
    pub filetype: Option<Filetype>,

    #[serde(flatten, default)]
    pub fields: Fields,
}

impl File {
    pub fn computed_filetype(&self) -> Option<Filetype> {
        if let Some(user_specified_filetype) = self.filetype.clone() {
            return Some(user_specified_filetype);
        }

        if let DataSource::FilePath(path_string) = &self.source {
            let path = Path::new(&path_string);
            let ext_str = path.extension()?.to_str()?;
            match String::from(ext_str).as_ref() {
                "html" => Some(Filetype::HTML),
                "htm" => Some(Filetype::HTML),
                "srt" => Some(Filetype::SRTSubtitle),
                "txt" => Some(Filetype::PlainText),
                _ => None,
            }
        } else {
            None
        }
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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Filetype {
    PlainText,
    SRTSubtitle,
    HTML,
}
