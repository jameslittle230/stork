use super::super::structs::Contents;
use super::super::structs::Entry;
use crate::common::Fields;

extern crate rust_stemmers;
use rust_stemmers::Algorithm;

pub(super) struct IntermediateEntry {
    pub(super) contents: Contents,
    pub(super) stem_algorithm: Option<Algorithm>,
    pub(super) title: String,
    pub(super) url: String,
    pub(super) fields: Fields,
}

impl From<&IntermediateEntry> for Entry {
    fn from(ie: &IntermediateEntry) -> Self {
        Entry {
            contents: ie.contents.get_full_text(),
            title: ie.title.clone(),
            url: ie.url.clone(),
            fields: ie.fields.clone(),
        }
    }
}
