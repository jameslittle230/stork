use crate::config::InputConfig;

use super::super::{AnnotatedWord, Contents};
use super::{WordListGenerationError, WordListGenerator};
use scraper::{Html, Selector};

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

        let word_list = document
            .select(&selector)
            .map(|elem_ref| {
                elem_ref.text().map(|w| {
                    w.to_string()
                        .split_whitespace()
                        .map(|str| str.to_string())
                        .collect::<Vec<String>>()
                })
            })
            .flatten() // Multiple text nodes within selectors
            .flatten() // Multiple words within text node
            .map(|word| AnnotatedWord {
                word,
                ..Default::default()
            })
            .collect::<Vec<AnnotatedWord>>();

        Ok(Contents { word_list })
    }
}

#[cfg(test)]
mod tests {
    use crate::config::InputConfig;

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

        assert_eq!(expected, computed);
    }

    #[test]
    fn test_html_content_extraction_with_multiple_selector_matches() {
        let expected = "This content should be indexed. This content is in a duplicate selector. It should also be indexed.";
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
                            <section class="yes"><p>This content is in a duplicate selector.</p><p>It should also be indexed.</p></section>
                        </main>
                    </body>
                </html>"#,
            ).ok().unwrap()
            .word_list
            .iter()
            .map(|aw| aw.word.clone())
            .collect::<Vec<String>>()
            .join(" ");

        assert_eq!(expected, computed);
    }

    #[test]
    fn test_html_content_extraction_from_inner_tags() {
        let expected = "This content should be indexed. This is another paragraph with inline text formatting . This is in a table cell.";
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
                            <section class="yes">
                                <p>This content should be indexed.</p>
                                <p>This is another paragraph with <strong><em>inline text</em>formatting</strong>.</p>
                                <div><img src="https://example.com/foo.png" /><table><tr><td>This is in a table cell.</td></tr></table></div>
                            </section>
                        </main>
                    </body>
                </html>"#,
            ).ok().unwrap()
            .word_list
            .iter()
            // .inspect(|aw| println!("{}", aw.word))
            .map(|aw| aw.word.clone().trim().to_string())
            .collect::<Vec<String>>()
            .join(" ");

        assert_eq!(expected, computed);
    }
}
