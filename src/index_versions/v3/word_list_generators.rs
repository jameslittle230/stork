use super::structs::{AnnotatedWord, Contents};
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
    fn create_word_list(&self, buffer: &str) -> Contents {
        let subs = srtparse::from_str(buffer).expect("Can't parse SRT file");
        let mut word_list: Vec<AnnotatedWord> = Vec::new();

        for sub in subs {
            for word in sub.text.split_whitespace() {
                word_list.push({
                    let mut fields = HashMap::new();
                    fields.insert("_srt_url_suffix".to_string(), format!("{}", sub.start_time));

                    AnnotatedWord {
                        word: word.to_string(),
                        fields,
                    }
                })
            }
        }

        Contents { word_list }
    }
}

pub(super) struct HTMLWordListGenerator {}

impl WordListGenerator for HTMLWordListGenerator {
    fn create_word_list(&self, _buffer: &str) -> Contents {
        Contents { word_list: vec![] }
    }
}
