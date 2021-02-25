use super::{
    super::structs::{AnnotatedWord, Contents},
    annotated_words_from_string::AnnotatedWordable,
};
use crate::common::InternalWordAnnotation;
use crate::config::{Filetype, InputConfig, SRTConfig, SRTTimestampFormat};
use std::fmt;

pub mod html_word_list_generator;
use html_word_list_generator::{HTMLWordListGenerator, MarkdownWordListGenerator};

#[derive(Debug)]
pub enum WordListGenerationError {
    InvalidSRT,
    FileNotFound,
    CannotDetermineFiletype,
    SelectorNotPresent(String),
    WebPageNotFetched,
    UnknownContentType,
    EmptyWordList,
}

impl fmt::Display for WordListGenerationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let desc: String = match self {
            WordListGenerationError::InvalidSRT => "SRT file could not be parsed".to_string(),
            WordListGenerationError::SelectorNotPresent(selector_string) => format!(
                "HTML selector `{}` is not present in the file",
                selector_string
            ),
            WordListGenerationError::FileNotFound => "The file could not be found".to_string(),
            WordListGenerationError::CannotDetermineFiletype => "Could not determine the filetype. Please use a known file extension or disambiguate the filetype within your configuration file".to_string(),
            WordListGenerationError::WebPageNotFetched => "The web page could not be fetched".to_string(),
            WordListGenerationError::UnknownContentType => "Content-Type is not present or invalid".to_string(),
            WordListGenerationError::EmptyWordList => "No words in word list".to_string(),
        };
        write!(f, "{}", desc)
    }
}

pub(super) fn returns_word_list_generator(filetype: &Filetype) -> Box<dyn WordListGenerator> {
    match filetype {
        Filetype::PlainText => Box::new(PlainTextWordListGenerator {}),
        Filetype::SRTSubtitle => Box::new(SRTWordListGenerator {}),
        Filetype::HTML => Box::new(HTMLWordListGenerator {}),
        Filetype::Markdown => Box::new(MarkdownWordListGenerator {}),
    }
}

pub(super) trait WordListGenerator {
    fn create_word_list(
        &self,
        config: &InputConfig,
        buffer: &str,
    ) -> Result<Contents, WordListGenerationError>;
}

pub(super) struct PlainTextWordListGenerator {}

impl WordListGenerator for PlainTextWordListGenerator {
    fn create_word_list(
        &self,
        _config: &InputConfig,
        buffer: &str,
    ) -> Result<Contents, WordListGenerationError> {
        Ok(Contents {
            word_list: buffer.make_annotated_words(),
        })
    }
}

pub(super) struct SRTWordListGenerator {}

impl WordListGenerator for SRTWordListGenerator {
    fn create_word_list(
        &self,
        config: &InputConfig,
        buffer: &str,
    ) -> Result<Contents, WordListGenerationError> {
        let subs = srtparse::from_str(buffer).map_err(|_e| WordListGenerationError::InvalidSRT)?;
        let mut word_list: Vec<AnnotatedWord> = Vec::new();

        for sub in subs {
            let mut annotated_words_for_this_sub =
                sub.text
                    .make_annotated_words_with_annotations(|_word, internal_annotations| {
                        internal_annotations.push(InternalWordAnnotation::SRTUrlSuffix(
                            SRTWordListGenerator::build_srt_url_time_suffix(
                                &sub.start_time,
                                &config.srt_config,
                            ),
                        ));
                    });

            word_list.append(&mut annotated_words_for_this_sub);
        }

        Ok(Contents { word_list })
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
