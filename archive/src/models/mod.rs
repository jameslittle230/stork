use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod config;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct StorkEntry {
    pub path: String,
    pub url: String,
    pub title: String,
    pub fields: Option<Vec<StorkField>>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
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
pub enum StorkResultOrAlias {
    Result(StorkResult),
    Alias(String),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StorkExcerpt {
    pub value: String,
    pub query_offset: u16,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StorkIndex {
    pub entries: Vec<StorkEntry>,
    pub results: HashMap<String, Vec<StorkResultOrAlias>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StorkOutput {
    pub entry: StorkEntry,
    pub result: StorkResult,
}
