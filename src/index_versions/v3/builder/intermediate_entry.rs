use super::super::structs::Contents;
use super::super::structs::Entry;
use crate::common::Fields;

extern crate rust_stemmers;
use rust_stemmers::Algorithm;

pub struct IntermediateEntry {
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

#[cfg(test)]
mod tests {
    use super::IntermediateEntry;
    use crate::index_versions::v3::structs::*;
    use std::collections::HashMap;

    #[test]
    fn convert_ie_to_entry() {
        let mut fields = HashMap::new();

        fields.insert("k1".to_string(), "v1".to_string());
        fields.insert("k2".to_string(), "v2".to_string());

        let intended = Entry {
            contents: "".to_string(),
            title: "My Title".to_string(),
            url: "https://example.com".to_string(),
            fields: fields.clone(),
        };

        let generated = Entry::from(&IntermediateEntry {
            contents: Contents { word_list: vec![] },
            stem_algorithm: None,
            title: "My Title".to_string(),
            url: "https://example.com".to_string(),
            fields: fields.clone(),
        });

        assert_eq!(generated.contents, intended.contents);
        assert_eq!(generated.title, intended.title);
        assert_eq!(generated.url, intended.url);
        assert_eq!(generated.fields, intended.fields);
    }
}
