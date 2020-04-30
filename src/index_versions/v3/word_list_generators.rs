use super::structs::{Contents, AnnotatedWord};
use std::collections::HashMap;

pub(super) trait WordListGenerator {
    fn create_word_list(&self, buffer: &str) -> Contents;
}

pub(super) struct PlainTextWordListGenerator {}

impl WordListGenerator for PlainTextWordListGenerator {
    fn create_word_list(&self, buffer: &str) -> Contents {
        Contents {
            word_list: buffer
                .split_whitespace()
                .map(|word| AnnotatedWord {
                    word: word.to_string(),
                    fields: HashMap::new(),
                })
                .collect(),
        }
    }
}

pub(super) struct SRTWordListGenerator {}

impl WordListGenerator for SRTWordListGenerator {
    fn create_word_list(&self, _buffer: &str) -> Contents {
        Contents { word_list: vec![] }
    }
}

pub(super) struct HTMLWordListGenerator {}

impl WordListGenerator for HTMLWordListGenerator {
    fn create_word_list(&self, _buffer: &str) -> Contents {
        Contents { word_list: vec![] }
    }
}
