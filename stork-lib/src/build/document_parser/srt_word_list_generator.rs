use crate::build::errors::DocumentReadError;
use crate::build::word_segmented_document::AnnotatedWord;

use crate::config::{Config, File, SRTConfig, SRTTimestampFormat};

use super::segment_words;

pub(crate) fn generate(
    config: &Config,
    _document_config: &File,
    contents: &str,
) -> Result<(String, Vec<AnnotatedWord>), DocumentReadError> {
    let subs = srtparse::from_str(&contents).map_err(|_e| DocumentReadError::InvalidSRT)?;

    let mut contents = String::new();
    let mut word_list: Vec<AnnotatedWord> = Vec::new();

    for sub in subs {
        let mut annotated_words = segment_words(&sub.text)
            .iter()
            .map(|indexed_word| AnnotatedWord {
                word: indexed_word.word.clone(),
                character_offset: indexed_word.character_offset,
                url_suffix: Some(build_srt_url_time_suffix(
                    &sub.start_time,
                    &config.input.srt_config,
                )),
            })
            .collect();

        contents.push_str(&sub.text);
        word_list.append(&mut annotated_words);
    }

    Ok((contents, word_list))
}

fn build_srt_url_time_suffix(time: &srtparse::Time, srt_config: &SRTConfig) -> String {
    let time_string = match srt_config.timestamp_format {
        SRTTimestampFormat::NumberOfSeconds => {
            ((time.hours) * 3600 + (time.minutes) * 60 + (time.seconds)).to_string()
        }
    };

    srt_config
        .timestamp_template_string
        .replace("{ts}", &time_string)
}
