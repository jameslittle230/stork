use thiserror::Error;

use crate::config::{SRTConfig, SRTTimestampFormat};
use crate::string_utils::split_into_normalized_words;

use super::annotated_word::AnnotatedWord;

#[derive(Debug, Clone, PartialEq, Error)]
pub(crate) enum SRTParseError {
    #[error("")]
    ReaderError,
}

pub(crate) fn generate(
    config: &crate::config::Config,
    file_index: usize,
    contents: &str,
) -> Result<(String, Vec<AnnotatedWord>), SRTParseError> {
    let subs = srtparse::from_str(&contents).map_err(|_| SRTParseError::ReaderError)?;
    generate_from_subs(subs, config)
}

fn generate_from_subs(
    subs: Vec<srtparse::Item>,
    config: &crate::config::Config,
) -> Result<(String, Vec<AnnotatedWord>), SRTParseError> {
    const SUB_SEPARATOR: &str = " ";
    let sub_sep_count: usize = SUB_SEPARATOR.len();

    let mut contents: Vec<String> = Vec::new();
    let mut word_list: Vec<AnnotatedWord> = Vec::new();

    let mut latest_sub_character_offset = 0;

    for sub in subs {
        let mut annotated_words: Vec<AnnotatedWord> = split_into_normalized_words(&sub.text)
            .iter()
            .map(|indexed_word| {
                AnnotatedWord::new(
                    indexed_word.word.clone(),
                    indexed_word.byte_offset + latest_sub_character_offset,
                    Some(build_srt_url_time_suffix(
                        &sub.start_time,
                        &config.input.srt_config,
                    )),
                )
            })
            .collect();

        latest_sub_character_offset = latest_sub_character_offset + sub.text.len() + sub_sep_count;
        contents.push(sub.text);
        word_list.append(&mut annotated_words);
    }

    Ok((contents.join(SUB_SEPARATOR), word_list))
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

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use srtparse::{Item, Time};

    use super::generate_from_subs;

    #[test]
    fn correctly_segments_words() {
        fn make_time(mins: u64) -> Time {
            Time {
                hours: 0,
                minutes: mins,
                seconds: 0,
                milliseconds: 0,
            }
        }

        let subs = vec![
            Item {
                pos: 0,
                start_time: make_time(0),
                end_time: make_time(1),
                text: "a bb-[ccc]; dddd".to_string(),
            },
            Item {
                pos: 1,
                start_time: make_time(1),
                end_time: make_time(2),
                text: "eeeee,ffff ggg hh".to_string(),
            },
            Item {
                pos: 1,
                start_time: make_time(2),
                end_time: make_time(3),
                text: "i          jj".to_string(),
            },
        ];
        let (computed_string, computed_annotated_words) =
            generate_from_subs(subs, &crate::config::Config::default()).unwrap();

        assert_eq!(
            "a bb-[ccc]; dddd eeeee,ffff ggg hh i          jj",
            computed_string
        );

        assert_eq!(
            vec!["a", "bb", "ccc", "dddd", "eeeee", "ffff", "ggg", "hh", "i", "jj"],
            computed_annotated_words
                .iter()
                .map(|aw| aw.word.clone())
                .collect::<Vec<String>>(),
        );

        assert_eq!(
            vec![0, 2, 6, 12, 17, 23, 28, 32, 35, 46],
            computed_annotated_words
                .iter()
                .map(|aw| aw.annotation.byte_offset)
                .collect::<Vec<usize>>(),
        );
    }
}
