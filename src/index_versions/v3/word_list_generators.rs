use super::structs::{AnnotatedWord, Contents};
use crate::config::{InputConfig, SRTConfig, SRTTimestampFormat};
use std::collections::HashMap;

pub(super) trait WordListGenerator {
    fn create_word_list(&self, config: &InputConfig, buffer: &str) -> Contents;
}

pub(super) struct PlainTextWordListGenerator {}

impl WordListGenerator for PlainTextWordListGenerator {
    fn create_word_list(&self, _config: &InputConfig, buffer: &str) -> Contents {
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
    fn create_word_list(&self, config: &InputConfig, buffer: &str) -> Contents {
        let subs = srtparse::from_str(buffer).expect("Can't parse SRT file");
        let mut word_list: Vec<AnnotatedWord> = Vec::new();

        for sub in subs {
            for word in sub.text.split_whitespace() {
                word_list.push({
                    let mut fields = HashMap::new();
                    if config.srt_config.timestamp_linking {
                        fields.insert(
                            "_srt_url_suffix".to_string(),
                            SRTWordListGenerator::build_srt_url_time_suffix(
                                &sub.start_time,
                                &config.srt_config,
                            ),
                        );
                    }

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

impl SRTWordListGenerator {
    fn build_srt_url_time_suffix(time: &srtparse::Time, srt_config: &SRTConfig) -> String {
        let time_string = match srt_config.timestamp_format {
            SRTTimestampFormat::NumberOfSeconds => ((time.hours as usize) * 3600
                + (time.minutes as usize) * 60
                + (time.seconds as usize))
                .to_string(),
        };

        srt_config
            .timestamp_template_string
            .replace("{ts}", &time_string)
    }
}

pub(super) struct HTMLWordListGenerator {}

impl WordListGenerator for HTMLWordListGenerator {
    fn create_word_list(&self, _config: &InputConfig, _buffer: &str) -> Contents {
        Contents { word_list: vec![] }
    }
}
