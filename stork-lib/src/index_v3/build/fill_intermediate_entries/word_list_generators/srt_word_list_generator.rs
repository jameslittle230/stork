use crate::config::SRTConfig;
use crate::config::SRTTimestampFormat;
use crate::index_v3::build::annotated_words_from_string::AnnotatedWordable;
use crate::index_v3::build::fill_intermediate_entries::ReaderConfig;
use crate::index_v3::AnnotatedWord;
use crate::index_v3::AnnotatedWordList;
use crate::InternalWordAnnotation;

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
        SRTTimestampFormat::MinutesAndSeconds => {
            if time.minutes > 0 {
                format!("{}m{}s", time.hours * 60 + time.minutes, time.seconds)
            } else {
                format!("{}s", time.seconds)
            }
        }
    };

    srt_config
        .timestamp_template_string
        .replace("{}", &time_string)
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::{
        config::SRTConfig,
        index_v3::build::fill_intermediate_entries::word_list_generators::srt_word_list_generator::build_srt_url_time_suffix,
    };

    #[test]
    fn it_formats_time_suffix_correctly() {
        let time = srtparse::Time {
            hours: 0,
            minutes: 16,
            seconds: 12,
            milliseconds: 0,
        };
        let config = SRTConfig {
            timestamp_linking: true,
            timestamp_template_string: "{}".to_string(),
            timestamp_format: crate::config::SRTTimestampFormat::NumberOfSeconds,
        };

        let computed = build_srt_url_time_suffix(&time, &config);
        let expected = "972";
        assert_eq!(computed, expected);
    }

    #[test]
    fn it_formats_time_suffix_correctly_minutes_and_seconds() {
        let time = srtparse::Time {
            hours: 0,
            minutes: 16,
            seconds: 12,
            milliseconds: 0,
        };
        let config = SRTConfig {
            timestamp_linking: true,
            timestamp_template_string: "{}".to_string(),
            timestamp_format: crate::config::SRTTimestampFormat::MinutesAndSeconds,
        };

        let computed = build_srt_url_time_suffix(&time, &config);
        let expected = "16m12s";
        assert_eq!(computed, expected);
    }

    #[test]
    fn it_formats_time_suffix_correctly_minutes_and_seconds_for_time_over_one_hour() {
        let time = srtparse::Time {
            hours: 1,
            minutes: 16,
            seconds: 12,
            milliseconds: 0,
        };
        let config = SRTConfig {
            timestamp_linking: true,
            timestamp_template_string: "{}".to_string(),
            timestamp_format: crate::config::SRTTimestampFormat::MinutesAndSeconds,
        };

        let computed = build_srt_url_time_suffix(&time, &config);
        let expected = "76m12s";
        assert_eq!(computed, expected);
    }
}
