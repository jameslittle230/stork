use stork_boundary::InternalWordAnnotation;
use stork_config::{SRTConfig, SRTTimestampFormat};

use crate::build::annotated_words_from_string::AnnotatedWordable;
use crate::build::fill_intermediate_entries::ReaderConfig;
use crate::AnnotatedWord;
use crate::AnnotatedWordList;

use super::ReadResult;
use super::WordListGenerationError;

pub fn generate(
    config: &ReaderConfig,
    read_result: &ReadResult,
) -> Result<AnnotatedWordList, WordListGenerationError> {
    let subs = srtparse::from_str(&read_result.buffer)
        .map_err(|_e| WordListGenerationError::InvalidSRT)?;
    let mut word_list: Vec<AnnotatedWord> = Vec::new();

    for sub in subs {
        let mut annotated_words_for_this_sub =
            sub.text
                .make_annotated_words_with_annotations(|_word, internal_annotations| {
                    internal_annotations.push(InternalWordAnnotation::UrlSuffix(
                        build_srt_url_time_suffix(&sub.start_time, &config.global.srt_config),
                    ));
                });

        word_list.append(&mut annotated_words_for_this_sub);
    }

    Ok(AnnotatedWordList { word_list })
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
