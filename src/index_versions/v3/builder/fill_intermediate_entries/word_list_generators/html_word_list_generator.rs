use crate::LatestVersion::structs::{AnnotatedWord, AnnotatedWordList};

use super::{ReadResult, ReaderConfig, WordListGenerationError};
use scraper::{Html, Selector};

pub fn generate(
    config: &ReaderConfig,
    read_result: &ReadResult,
) -> Result<AnnotatedWordList, WordListGenerationError> {
    let document = Html::parse_document(&read_result.buffer);

    let selector_string = (&config
        .file
        .html_selector_override
        .as_deref()
        .or(config.global.html_selector.as_deref()))
        .unwrap_or_else(|| "main");
    let selector = Selector::parse(selector_string).unwrap();

    // We could just check to see if the outputted vec at the end of the
    // data chain is empty, but I explicitly want to avoid throwing this error
    // if the selector _is_ present but there are no words.
    let selector_match_in_document_count = document.select(&selector).count();
    if selector_match_in_document_count == 0 {
        return Err(WordListGenerationError::SelectorNotPresent(
            selector_string.to_string(),
        ));
    }

    let word_list = document
        .select(&selector)
        .flat_map(|elem_ref| {
            elem_ref.text().map(|w| {
                w.to_string()
                    .split_whitespace()
                    .map(ToString::to_string)
                    .collect::<Vec<String>>()
            })
        })
        .flatten() // Multiple words within text node
        .map(|word| AnnotatedWord {
            word,
            ..AnnotatedWord::default()
        })
        .collect::<Vec<AnnotatedWord>>();

    Ok(AnnotatedWordList { word_list })
}

#[cfg(test)]
mod tests {
    use super::{generate, ReadResult, ReaderConfig, WordListGenerationError};
    use crate::{
        config::{File, Filetype, InputConfig},
        LatestVersion::structs::AnnotatedWordList,
    };

    fn reader_config_from_html_selector(selector: Option<&str>) -> ReaderConfig {
        let mut file = File::default();
        file.html_selector_override = selector.map(ToString::to_string);
        ReaderConfig {
            global: InputConfig::default(),
            file,
        }
    }

    fn read_result_from_string(str: &str) -> ReadResult {
        ReadResult {
            buffer: str.to_string(),
            filetype: Some(Filetype::HTML),
            frontmatter_fields: None,
        }
    }

    fn extract_text(result: Result<AnnotatedWordList, WordListGenerationError>) -> String {
        return result
            .ok()
            .unwrap()
            .word_list
            .iter()
            .map(|aw| aw.word.clone())
            .collect::<Vec<String>>()
            .join(" ");
    }

    fn run_html_parse_test(expected: &str, selector: Option<&str>, html: &str) {
        let computed = extract_text(generate(
            &reader_config_from_html_selector(selector),
            &read_result_from_string(html),
        ));

        assert_eq!(expected, computed)
    }

    #[test]
    fn test_basic_html_content_extraction() {
        run_html_parse_test(
            "This is some text",
            None,
            r#"
        <html>
            <head></head>
            <body><h1>This is a title</h1><main><p>This is some text</p></main></body>
        </html>"#,
        );
    }

    //     #[test]
    //     fn test_html_content_extraction_with_custom_selector() {
    //         let expected = "This content should be indexed";
    //         let computed: String = (HTMLWordListGenerator {})
    //             .create_word_list(
    //                 &InputConfig {
    //                     html_selector: Some(".yes".to_string()),
    //                     ..Default::default()
    //                 },
    //                 r#"
    //                 <html>
    //                     <head></head>
    //                     <body>
    //                         <h1>This is a title</h1>
    //                         <main>
    //                             <section class="no"><p>Stork should not recognize this text</p></section>
    //                             <section class="yes"><p>This content should be indexed</p></section>
    //                         </main>
    //                     </body>
    //                 </html>"#,
    //             ).ok().unwrap()
    //             .word_list
    //             .iter()
    //             .map(|aw| aw.word.clone())
    //             .collect::<Vec<String>>()
    //             .join(" ");

    //         assert_eq!(expected, computed);
    //     }

    //     #[test]
    //     fn test_selector_not_present() {
    //         let computed = (HTMLWordListGenerator {})
    //             .create_word_list(
    //                 &InputConfig {
    //                     html_selector: Some(".yes".to_string()),
    //                     ..Default::default()
    //                 },
    //                 r#"
    //                 <html>
    //                     <head></head>
    //                     <body>
    //                         <h1>This is a title</h1>
    //                         <main>
    //                             <section class="no"><p>Stork should not recognize this text</p></section>
    //                         </main>
    //                     </body>
    //                 </html>"#,
    //             ).is_err();

    //         assert_eq!(true, computed);
    //     }

    //     #[test]
    //     fn test_selector_present_but_empty_contents() {
    //         let computed = (HTMLWordListGenerator {})
    //             .create_word_list(
    //                 &InputConfig {
    //                     html_selector: Some(".yes".to_string()),
    //                     ..Default::default()
    //                 },
    //                 r#"
    //                 <html>
    //                     <head></head>
    //                     <body>
    //                         <h1>This is a title</h1>
    //                         <main>
    //                             <section class="no"><p>Stork should not recognize this text</p></section>
    //                             <section class="yes"><p></p></section>
    //                         </main>
    //                     </body>
    //                 </html>"#,
    //             ).ok().unwrap() // it shouldn't panic here! if it does the test has failed
    //             .word_list.len();

    //         assert_eq!(0, computed);
    //     }

    //     #[test]
    //     fn test_html_content_extraction_with_multiple_selector_matches() {
    //         let expected = "This content should be indexed. This content is in a duplicate selector. It should also be indexed.";
    //         let computed: String = create_word_list(
    //                 &InputConfig {
    //                     html_selector: Some(".yes".to_string()),
    //                     ..Default::default()
    //                 },
    //                 r#"
    //                 <html>
    //                     <head></head>
    //                     <body>
    //                         <h1>This is a title</h1>
    //                         <main>
    //                             <section class="no"><p>Stork should not recognize this text</p></section>
    //                             <section class="yes"><p>This content should be indexed.</p></section>
    //                             <section class="yes"><p>This content is in a duplicate selector.</p><p>It should also be indexed.</p></section>
    //                         </main>
    //                     </body>
    //                 </html>"#,
    //             ).ok().unwrap()
    //             .word_list
    //             .iter()
    //             .map(|aw| aw.word.clone())
    //             .collect::<Vec<String>>()
    //             .join(" ");

    //         assert_eq!(expected, computed);
    //     }

    //     #[test]
    //     fn test_html_content_extraction_from_inner_tags() {
    //         let expected = "This content should be indexed. This is another paragraph with inline text formatting . This is in a table cell.";
    //         let computed: String = (HTMLWordListGenerator {})
    //             .create_word_list(
    //                 &InputConfig {
    //                     html_selector: Some(".yes".to_string()),
    //                     ..Default::default()
    //                 },
    //                 r#"
    // <html>
    //     <head></head>
    //     <body>
    //         <h1>This is a title</h1>
    //         <main>
    //             <section class="no"><p>Stork should not recognize this text</p></section>
    //             <section class="yes">
    //                 <p>This content should be indexed.</p>
    //                 <p>This is another paragraph with <strong><em>inline text</em>formatting</strong>.</p>
    //                 <div><img src="https://example.com/foo.png" /><table><tr><td>This is in a table cell.</td></tr></table></div>
    //             </section>
    //         </main>
    //     </body>
    // </html>
    //                 "#,
    //             ).ok().unwrap()
    //             .word_list
    //             .iter()
    //             // .inspect(|aw| println!("{}", aw.word))
    //             .map(|aw| aw.word.clone().trim().to_string())
    //             .collect::<Vec<String>>()
    //             .join(" ");

    //         assert_eq!(expected, computed);
    //     }

    //     #[test]
    //     fn test_markdown() {
    //         let expected = "This is a title Stork should recognize this text This content should be indexed. This is another paragraph with inline text formatting . This is a link. Goodbye!";
    //         let computed: String = (MarkdownWordListGenerator {})
    //             .create_word_list(
    //                 &InputConfig::default(),
    //                 r#"
    // # This is a title

    // Stork should recognize this text

    // - This content should be indexed.
    // - This is another paragraph with **_inline text_formatting**.
    // - [This is a link.](https://example.com)

    // Goodbye!
    //                 "#,
    //             )
    //             .ok()
    //             .unwrap()
    //             .word_list
    //             .iter()
    //             // .inspect(|aw| println!("{}", aw.word))
    //             .map(|aw| aw.word.clone().trim().to_string())
    //             .collect::<Vec<String>>()
    //             .join(" ");

    //         assert_eq!(expected, computed);
    //     }
}
