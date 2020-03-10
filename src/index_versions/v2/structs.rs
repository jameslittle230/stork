use crate::Fields;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type EntryIndex = usize;
pub type AliasTarget = String;
pub type Score = u8;

pub const HALF_U8: u8 = u8::max_value() / 2;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(super) struct Entry {
    pub(super) contents: String,
    pub(super) title: String,
    pub(super) url: String,
    pub(super) fields: Fields,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(super) struct SearchResult {
    pub(super) excerpts: Vec<Excerpt>,
    pub(super) score: Score,
}

impl SearchResult {
    pub(super) fn new() -> SearchResult {
        SearchResult {
            excerpts: vec![],
            score: HALF_U8,
        }
    }
}

// #[derive(Serialize, Deserialize, Clone, Debug)]
// pub(super) struct RangeInText {
//     pub(super) begin: usize,
//     pub(super) end: usize,
// }

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(super) struct Excerpt {
    pub(super) word_index: usize,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(super) struct Container {
    pub(super) results: HashMap<EntryIndex, SearchResult>,
    pub(super) aliases: HashMap<AliasTarget, Score>,
}

impl Container {
    pub fn new() -> Container {
        Container {
            results: HashMap::new(),
            aliases: HashMap::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Index {
    pub(super) entries: Vec<Entry>,
    pub(super) queries: HashMap<String, Container>,
}
