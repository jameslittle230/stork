use crate::Fields;
use super::StemmingConfig;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
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

impl Default for File {
    fn default() -> Self {
        File {
            title: "".to_string(),
            url: "".to_string(),
            source: DataSource::Contents("".to_string()),
            id: None,
            stemming_override: None,
            filetype: None,
            fields: HashMap::new()
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum DataSource {

    #[serde(rename = "contents")]
    Contents(String),

    #[serde(rename = "src_url")]
    URL(String),

    #[serde(rename = "path")]
    FilePath(String)
}


#[derive(Serialize, Deserialize, Debug)]
pub enum Filetype {
    PlainText,
    SRTSubtitle,
}