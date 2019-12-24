use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Debug)]
pub struct StorkEntry {
    pub path: String,
    pub url: String,
    pub title: String,
    pub fields: Option<Vec<StorkField>>,
}

#[derive(Clone, Deserialize, Debug)]
pub struct StorkField {
    key: String,
    value: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StorkResult {
    pub file_index: u32,
    pub excerpts: Vec<StorkExcerpt>,
    pub score: u8,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StorkExcerpt {
    pub value: String,
    pub query_offset: u16,
}
