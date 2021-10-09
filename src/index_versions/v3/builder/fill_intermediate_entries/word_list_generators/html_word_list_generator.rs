use std::collections::HashMap;

use crate::common::InternalWordAnnotation;
use crate::LatestVersion::structs::{AnnotatedWord, AnnotatedWordList};
use kuchiki::{traits::*, ElementData, NodeDataRef};

use super::{ReadResult, ReaderConfig, WordListGenerationError};

pub fn generate(
    config: &ReaderConfig,
    read_result: &ReadResult,
) -> Result<AnnotatedWordList, WordListGenerationError> {
    let document = kuchiki::parse_html().one(read_result.buffer.clone());

    let selector: &str = {
        config
            .file
            .html_selector_override
            .as_ref()
            .or(config.global.html_selector.as_ref())
            .map(|a| a.as_str())
            .unwrap_or("main")
    };

    let exclude_selector: Option<&str> = {
        config
            .file
            .exclude_html_selector_override
            .as_ref()
            .or(config.global.exclude_html_selector.as_ref())
            .map(|a| a.as_str())
    };

    if let Ok(css_matches) = document.select(selector) {
        let mut word_list: Vec<AnnotatedWord> = vec![];
        let mut latest_id: Option<String> = None;

        let matches_vec: Vec<NodeDataRef<ElementData>> = css_matches.into_iter().collect();

        if matches_vec.is_empty() {
            return Err(WordListGenerationError::SelectorNotPresent(
                selector.to_string(),
            ));
        }

        for css_match in matches_vec {
            let as_node = css_match.as_node();

            if let Some(exclude_selector) = exclude_selector {
                if let Ok(excluded_elements) = as_node.select(exclude_selector) {
                    for excluded_element in excluded_elements {
                        excluded_element.as_node().detach();
                    }
                }
            }

            for elem in as_node.traverse_inclusive() {
                println!("{:?}", elem);
                if let kuchiki::iter::NodeEdge::Start(node_ref) = elem {
                    let contents: String = (|| {
                        let mut output = "".to_string();
                        if let Some(element_data) = node_ref.as_element() {
                            if let Some(id) = element_data.attributes.borrow().get("id") {
                                latest_id = Some(id.to_string());
                            }

                            let contentful_attrs = vec!["title", "alt"];

                            for attr in contentful_attrs {
                                if let Some(value) = element_data.attributes.borrow().get(attr) {
                                    output.push_str(value);
                                }
                            }
                        }

                        if let Some(refcell) = node_ref.as_text() {
                            output.push_str(refcell.borrow().as_ref());
                        }
                        return output.trim().to_string();
                    })();

                    if !contents.is_empty() {
                        let mut annotated_words: Vec<AnnotatedWord> = contents
                            .split_whitespace()
                            .map(ToString::to_string)
                            .map(|word| AnnotatedWord {
                                word,
                                internal_annotations: {
                                    if let Some(latest_id) = latest_id.clone() {
                                        vec![InternalWordAnnotation::NearestHtmlId(latest_id)]
                                    } else {
                                        vec![]
                                    }
                                },
                                fields: HashMap::default(),
                            })
                            .collect();

                        word_list.append(&mut annotated_words);
                    }
                }
            }
        }

        if word_list.is_empty() {
            return Err(WordListGenerationError::EmptyWordList);
        } else {
            return Ok(AnnotatedWordList { word_list });
        }
    }

    Err(WordListGenerationError::SelectorNotPresent(
        selector.to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::{generate, ReadResult, ReaderConfig, WordListGenerationError};
    use crate::{
        config::{File, Filetype, InputConfig},
        LatestVersion::structs::AnnotatedWordList,
    };

    fn reader_config_from_html_selectors(
        include: Option<&str>,
        exclude: Option<&str>,
    ) -> ReaderConfig {
        let mut file = File::default();
        file.html_selector_override = include.map(ToString::to_string);
        file.exclude_html_selector_override = exclude.map(ToString::to_string);
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

    fn run_html_parse_test(
        expected: &str,
        include: Option<&str>,
        exclude: Option<&str>,
        html: &str,
    ) {
        let computed = extract_text(generate(
            &reader_config_from_html_selectors(include, exclude),
            &read_result_from_string(html),
        ));

        assert_eq!(
            expected, computed,
            "expected: {}\ncomputed: {}",
            expected, computed
        )
    }

    #[test]
    fn test_basic_html_content_extraction() {
        run_html_parse_test(
            "This is some text",
            None,
            None,
            r#"
        <html>
            <head></head>
            <body><h1>This is a title</h1><main><p>This is some text</p></main></body>
        </html>"#,
        );
    }

    #[test]
    fn test_html_content_extraction_with_custom_selector() {
        run_html_parse_test(
            "This content should be indexed",
            Some(".yes"),
            None,
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
        )
    }

    #[test]
    fn test_html_content_extraction_with_excluded_selector() {
        run_html_parse_test(
            "This content should be indexed This content should also be indexed",
            Some(".yes"),
            Some(".no"),
            r#"
        <html>
            <head></head>
            <body>
                <h1>This is a title</h1>
                <main>
                    <section class="yes" id="first">
                        <p>This content should be indexed</p>
                        <p id="second">This content should also be indexed</p>
                        <p class="no">This content should not be indexed</p>
                    </section>
                </main>
            </body>
        </html>"#,
        )
    }

    #[test]
    fn test_selector_not_present() {
        let computed = generate(
                &reader_config_from_html_selectors(Some(".yes"), Some(".no")),
                &read_result_from_string(r#"
                <html>
                    <head></head>
                    <body>
                        <h1>This is a title</h1>
                        <main>
                            <section class="no"><p>Stork should not recognize this text</p></section>
                        </main>
                    </body>
                </html>"#)).is_err();

        assert!(computed);
    }

    #[test]
    fn test_selector_present_but_empty_contents() {
        let computed = generate(
                &reader_config_from_html_selectors(Some(".yes"), Some(".no")),
                &read_result_from_string(r#"
                    <html>
                        <head></head>
                        <body>
                            <h1>This is a title</h1>
                            <main>
                                <section class="no"><p>Stork should not recognize this text</p></section>
                                <section class="yes"><p></p></section>
                            </main>
                        </body>
                    </html>"#)).unwrap_err();

        assert_eq!(WordListGenerationError::EmptyWordList, computed);
    }

    #[test]
    fn test_html_content_extraction_with_multiple_selector_matches() {
        run_html_parse_test(
            "This content should be indexed. This content is in a duplicate selector. It should also be indexed.",
            Some(".yes"),
            None,
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
            </html>"#)
    }

    #[test]
    fn test_img_alt_text_extraction() {
        run_html_parse_test(
            "This content should be indexed. This is a random text node that should be picked up! A nice bird! 2004-era interactivity! This content is in a duplicate selector. It should also be indexed.",
            Some(".yes"),
            None,
            r#"
            <html>
                <head></head>
                <body>
                    <h1>This is a title</h1>
                    <main>
                        <section class="no"><p>Stork should not recognize this text</p></section>
                        This is a random text node that should not be picked up!
                        <section class="yes">
                            <p>This content should be indexed.</p>
                            This is a random text node that should be picked up!
                            <img src="https://stork-search.net/logo.svg" alt="A nice bird!"></img>
                            <applet src="https://stork-search.net/logo.svg" alt="2004-era interactivity!"></applet>
                        </section>
                        <section class="yes"><p>This content is in a duplicate selector.</p><p>It should also be indexed.</p></section>
                    </main>
                </body>
            </html>"#)
    }

    #[test]
    fn test_index_blocklist() {
        run_html_parse_test(
            "Article content More article content",
            Some("article"),
            Some(".no-index"),
            r#"<main>
        <aside>...</aside>
        <article>
          <p>Article content</p>
          <div class="no-index">This shouldn't be indexed</div>
          <p>More article content</p>
        </article>
      </main>"#,
        )
    }

    #[test]
    fn test_self_closing_tag_behavior() {
        /*
         * This test originally had both the `img` and the `applet` tag as
         * self-closing. However, the `applet` tag is not allowed to self-
         * close per MDN. The test passes when only the img tag is self-
         * closing, but does not pass when the applet tag is as well.
         */

        run_html_parse_test(
            "This content should be indexed. This is a random text node that should be picked up! A nice bird! 2004-era interactivity! This content is in a duplicate selector. It should also be indexed.",
            Some(".yes"),
            None,
            r#"
            <html>
                <head></head>
                <body>
                    <h1>This is a title</h1>
                    <main>
                        <section class="no"><p>Stork should not recognize this text</p></section>
                        This is a random text node that should not be picked up!
                        <section class="yes">
                            <p>This content should be indexed.</p>
                            This is a random text node that should be picked up!
                            <img src="https://stork-search.net/logo.svg" alt="A nice bird!" />
                            <applet src="https://stork-search.net/logo.svg" alt="2004-era interactivity!"></applet>
                        </section>
                        <section class="yes"><p>This content is in a duplicate selector.</p><p>It should also be indexed.</p></section>
                    </main>
                </body>
            </html>"#)
    }

    #[test]
    fn test_html_content_extraction_from_inner_tags() {
        let expected = "This content should be indexed. This is another paragraph with inline text formatting . This is in a table cell.";
        run_html_parse_test(
            expected,
            Some(".yes"),
            None,
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
        );
    }
}
