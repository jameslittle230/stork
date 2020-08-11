use super::super::structs::{AnnotatedWord, Contents};
use crate::common::InternalWordAnnotation;
use crate::config::{Filetype, InputConfig, SRTConfig, SRTTimestampFormat};
use scraper::{Html, Selector};
use std::fmt;

pub enum WordListGenerationError {
    InvalidSRT,
    InvalidHTML,
    SelectorNotPresent,
}

impl fmt::Display for WordListGenerationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let desc: String = match self {
            WordListGenerationError::InvalidHTML => "Invalid HTML",
            WordListGenerationError::InvalidSRT => "Invalid SRT",
            WordListGenerationError::SelectorNotPresent => "HTML selector not present in contents",
        }
        .to_string();
        write!(f, "{}", desc)
    }
}

pub(super) fn returns_word_list_generator(filetype: &Filetype) -> Box<dyn WordListGenerator> {
    match filetype {
        Filetype::PlainText => Box::new(PlainTextWordListGenerator {}),
        Filetype::SRTSubtitle => Box::new(SRTWordListGenerator {}),
        Filetype::HTML => Box::new(HTMLWordListGenerator {}),
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
            word_list: buffer
                .split_whitespace()
                .map(|word| AnnotatedWord {
                    word: word.to_string(),
                    ..Default::default()
                })
                .collect(),
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
            for word in sub.text.split_whitespace() {
                word_list.push({
                    let mut internal_annotations: Vec<InternalWordAnnotation> = vec![];
                    if config.srt_config.timestamp_linking {
                        internal_annotations.push(InternalWordAnnotation::SRTUrlSuffix(
                            SRTWordListGenerator::build_srt_url_time_suffix(
                                &sub.start_time,
                                &config.srt_config,
                            ),
                        ));
                    }

                    AnnotatedWord {
                        word: word.to_string(),
                        internal_annotations,
                        ..Default::default()
                    }
                })
            }
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

pub(super) struct HTMLWordListGenerator {}

impl WordListGenerator for HTMLWordListGenerator {
    fn create_word_list(
        &self,
        config: &InputConfig,
        buffer: &str,
    ) -> Result<Contents, WordListGenerationError> {
        let document = Html::parse_document(buffer);
        let selector_string = (config.html_selector.clone()).unwrap_or_else(|| "main".to_string());
        let selector = Selector::parse(selector_string.as_str()).unwrap();
        let selector_contents = document
            .select(&selector)
            .next()
            .ok_or_else(|| WordListGenerationError::SelectorNotPresent)?;
        let text = selector_contents
            .text()
            .collect::<Vec<_>>()
            .iter()
            .map(|word| AnnotatedWord {
                word: word.to_string(),
                ..Default::default()
            })
            .collect();

        Ok(Contents { word_list: text })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_basic_html_content_extraction() {
        let expected = "This is some text";
        let computed: String = (HTMLWordListGenerator {})
            .create_word_list(
                &InputConfig::default(),
                r#"
                <html>
                    <head></head>
                    <body><h1>This is a title</h1><main><p>This is some text</p></main></body>
                </html>"#,
            )
            .ok()
            .unwrap()
            .word_list
            .iter()
            .map(|aw| aw.word.clone())
            .collect::<Vec<String>>()
            .join(" ");

        assert!(expected == computed);
    }

    #[test]
    fn test_html_content_extraction_with_custom_selector() {
        let expected = "This content should be indexed";
        let computed: String = (HTMLWordListGenerator {})
            .create_word_list(
                &InputConfig {
                    html_selector: Some(".yes".to_string()),
                    ..Default::default()
                },
                r#"
                <html>
                    <head></head>
                    <body>
                        <h1>This is a title</h1>
                        <main>
                            <section class="no"><p>Stork should not recognize this text</p></section>
                            <section class="yes"><p>This content should be indexed</p></section>
                        </main>
                    </body>
                </html>"#,
            ).ok().unwrap()
            .word_list
            .iter()
            .map(|aw| aw.word.clone())
            .collect::<Vec<String>>()
            .join(" ");

        assert!(expected == computed);
    }

    #[test]
    #[ignore = "Not implemented yet"]
    fn test_html_content_extraction_with_multiple_selector_matches() {
        let expected = "This content should be indexed. This content is in a duplicate selector.";
        let computed: String = (HTMLWordListGenerator {})
            .create_word_list(
                &InputConfig {
                    html_selector: Some(".yes".to_string()),
                    ..Default::default()
                },
                r#"
                <html>
                    <head></head>
                    <body>
                        <h1>This is a title</h1>
                        <main>
                            <section class="no"><p>Stork should not recognize this text</p></section>
                            <section class="yes"><p>This content should be indexed.</p></section>
                            <section class="yes"><p>This content is in a duplicate selector.</p></section>
                        </main>
                    </body>
                </html>"#,
            ).ok().unwrap()
            .word_list
            .iter()
            .map(|aw| aw.word.clone())
            .collect::<Vec<String>>()
            .join(" ");

        assert!(expected == computed);
    }
}
