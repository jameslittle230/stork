use kuchiki::{traits::TendrilSink, ElementData, NodeDataRef};
use thiserror::Error;

use crate::string_utils::split_into_normalized_words;

use crate::build_config::HTMLConfig;

use super::annotated_word::AnnotatedWord;

#[derive(Debug, Clone, PartialEq, Error)]
pub(crate) enum HtmlParseError {
    #[error("The selector `{0}` is not present in the HTML document.")]
    SelectorNotPresent(String),
}

pub(crate) fn parse(
    contents: &str,
    config: &HTMLConfig,
) -> Result<(String, Vec<AnnotatedWord>), HtmlParseError> {
    let html_document = kuchiki::parse_html().one(contents);

    let selectors = config.included_selectors.join(", ");
    let excluded_selectors = config.excluded_selectors.join(", ");

    if let Ok(document_matches) = html_document.select(&selectors) {
        let mut word_list: Vec<AnnotatedWord> = vec![];
        let mut latest_id: Option<String> = None;
        let mut document_contents: Vec<String> = vec![];

        let matches_vec: Vec<NodeDataRef<ElementData>> = document_matches.collect();

        if matches_vec.is_empty() {
            return Err(HtmlParseError::SelectorNotPresent(selectors.to_string()));
        }

        'matches: for css_match in matches_vec {
            let matching_node = css_match.as_node();

            if let Ok(exclusion_selection) = matching_node.select(&excluded_selectors) {
                // Kuchiki doesn't like it if you mutate the tree while iterating over it,
                // so instead of iterating over the .select() result, let's first collect
                // all nodes in our iterator, then remove them after the fact.
                let excluded_nodes: Vec<NodeDataRef<_>> = exclusion_selection.into_iter().collect();

                for node in excluded_nodes {
                    // If the outermost node we matched on also matches the exclusion selector,
                    // skip processing the entire matched node.
                    if node.as_node() == matching_node {
                        continue 'matches;
                    }
                    node.as_node().detach();
                }
            }

            for elem in matching_node.traverse_inclusive() {
                if let kuchiki::iter::NodeEdge::Start(node_ref) = elem {
                    // Build up string that represents the contents of this DOM node
                    let contents: String = {
                        let mut output: Vec<String> = vec![];
                        if let Some(element_data) = node_ref.as_element() {
                            if config.save_nearest_id {
                                if let Some(id) = element_data.attributes.borrow().get("id") {
                                    latest_id = Some(id.to_string());
                                }
                            }

                            let contentful_attrs = vec!["title", "alt"];

                            for attr in contentful_attrs {
                                if let Some(value) = element_data.attributes.borrow().get(attr) {
                                    output.push(value.trim().to_string());
                                }
                            }
                        }

                        if let Some(refcell) = node_ref.as_text() {
                            output.push(refcell.borrow().trim().to_string());
                        }

                        output.join(" ")
                    };

                    if !contents.is_empty() {
                        document_contents.push(contents.clone());
                        let words = split_into_normalized_words(&contents);

                        let mut annotated_words = words
                            .into_iter()
                            .map(|indexed_word| {
                                AnnotatedWord::new(
                                    indexed_word.word,
                                    indexed_word.byte_offset,
                                    latest_id.clone().map(|id| format!("#{id}")),
                                )
                            })
                            .collect();

                        word_list.append(&mut annotated_words);
                    }
                }
            }
        }

        return Ok((document_contents.join(" "), word_list));
    }

    Err(HtmlParseError::SelectorNotPresent(selectors))
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::HtmlParseError;
    use crate::build_config::HTMLConfig;

    #[test]
    fn basic_text_extraction() {
        let html = r#"
            <html>
                <body>
                    <div id="content">
                        <p>Some text</p>
                        <p>Some more text</p>
                    </div>
                </body>
            </html>"#;

        let config = HTMLConfig {
            included_selectors: vec!["#content".to_string()],
            excluded_selectors: vec![],
            title_selector: "h1".to_string(),
            save_nearest_id: true,
        };

        let (text, words) = super::parse(html, &config).unwrap();

        assert_eq!(text, "Some text Some more text");
    }

    #[test]
    fn selective_content_extraction() {
        let html = r#"
            <html>
                <body>
                    <div id="content">
                        <p class="yes">Some text</p>
                        <p>Some more text</p>
                    </div>
                </body>
            </html>"#;

        let config = HTMLConfig {
            included_selectors: vec![".yes".to_string()],
            excluded_selectors: vec![],
            title_selector: "h1".to_string(),
            save_nearest_id: true,
        };

        let (text, words) = super::parse(html, &config).unwrap();

        assert_eq!(text, "Some text");
    }

    #[test]
    fn content_exclusion() {
        let html = r#"
            <html>
                <body>
                    <div class="yes">
                        <p>Some text</p>
                        <p class="no">Some more text</p>
                    </div>
                </body>
            </html>"#;

        let config = HTMLConfig {
            included_selectors: vec![".yes".to_string()],
            excluded_selectors: vec![".no".to_string()],
            title_selector: "h1".to_string(),
            save_nearest_id: true,
        };

        let (text, words) = super::parse(html, &config).unwrap();

        assert_eq!(text, "Some text");
    }

    #[test]
    fn selective_content_extraction_multiple_included_selectors() {
        let html = r#"
            <html>
                <body>
                    <div id="content">
                        <p class="yes-1">Some text</p>
                        <p>Some more text</p>
                        <p class="yes-2">Even more text</p>
                        <p>This is some text</p>
                    </div>
                </body>
            </html>"#;

        let config = HTMLConfig {
            included_selectors: vec![".yes-1".to_string(), ".yes-2".to_string()],
            excluded_selectors: vec![],
            title_selector: "h1".to_string(),
            save_nearest_id: true,
        };

        let (text, words) = super::parse(html, &config).unwrap();

        assert_eq!(text, "Some text Even more text");
    }

    #[test]
    fn selective_content_extraction_multiple_excluded_selectors() {
        let html = r#"
            <html>
                <body>
                    <div id="content">
                        <p class="yes-1">Some text</p>
                        <p>Some more text</p>
                        <p class="yes-2">Even more text</p>
                        <p>This is some text</p>
                    </div>
                </body>
            </html>"#;

        let config = HTMLConfig {
            included_selectors: vec!["#content".to_string()],
            excluded_selectors: vec![".yes-1".to_string(), ".yes-2".to_string()],
            title_selector: "h1".to_string(),
            save_nearest_id: true,
        };

        let (text, words) = super::parse(html, &config).unwrap();

        assert_eq!(text, "Some more text This is some text");
    }

    #[test]
    fn excluded_selectors_override_included_selectors() {
        let html = r#"
            <html>
                <body>
                    <div id="content">
                        <p class="yes no">Some text</p>
                        <p class="yes">Some more text</p>
                        <p class="no">Even more text</p>
                        <p>This is some text</p>
                    </div>
                </body>
            </html>"#;

        let config = HTMLConfig {
            included_selectors: vec![".yes".to_string()],
            excluded_selectors: vec![".no".to_string()],
            title_selector: "h1".to_string(),
            save_nearest_id: true,
        };

        let (text, words) = super::parse(html, &config).unwrap();

        assert_eq!(text, "Some more text");
    }

    #[test]
    fn excluded_selectors_override_nested_included_selectors() {
        let html = r#"
        <html>
            <body>
                <h1>This is a title</h1>
                <main>
                    <div class="yes">
                        <p>This content should be indexed</p>
                        <p class="no">This content should not be indexed</p>
                        <p class="no yes">This content should not be indexed. The exclusion selector should override the inclusion selector when the inclusion selector is nested inside another inclusion selector</p>
                    </div>
                </main>
            </body>
        </html>"#;

        let config = HTMLConfig {
            included_selectors: vec![".yes".to_string()],
            excluded_selectors: vec![".no".to_string()],
            title_selector: "h1".to_string(),
            save_nearest_id: true,
        };

        let (text, words) = super::parse(html, &config).unwrap();

        assert_eq!(text, "This content should be indexed");
    }

    #[test]
    fn selector_not_present_error() {
        let html = r#"
        <html>
            <body>
                <div class="yes">
                    <p>This content should be indexed</p>
                    <p class="no">This content should not be indexed</p>
                    <p class="no yes">This content should not be indexed. The exclusion selector should override the inclusion selector when the inclusion selector is nested inside another inclusion selector</p>
                </div>
            </body>
        </html>"#;

        let config = HTMLConfig {
            included_selectors: vec![".not-present".to_string()],
            excluded_selectors: vec![".no".to_string()],
            title_selector: "h1".to_string(),
            save_nearest_id: true,
        };

        let error = super::parse(html, &config).err().unwrap();

        assert_eq!(
            error,
            HtmlParseError::SelectorNotPresent(".not-present".to_string())
        );
    }

    #[test]
    fn excluded_selector_not_present_is_not_an_error() {
        let html = r#"
        <html>
            <body>
                <div class="yes">
                    <p>This content should be indexed</p>
                    <p class="no">This content should not be indexed</p>
                    <p class="no yes">This content should not be indexed. The exclusion selector should override the inclusion selector when the inclusion selector is nested inside another inclusion selector</p>
                </div>
            </body>
        </html>"#;

        let config = HTMLConfig {
            included_selectors: vec![".yes".to_string()],
            excluded_selectors: vec![".not-present".to_string()],
            title_selector: "h1".to_string(),
            save_nearest_id: true,
        };

        let is_ok = super::parse(html, &config).is_ok();

        assert!(is_ok);
    }

    #[test]
    fn empty_word_list_trim() {
        let html = r#"
            <html>
                <body>
                    <div id="content">
                        <p class="yes"><span> <i></i></span></p>
                        <p>Some more text</p>
                    </div>
                </body>
            </html>"#;

        let config = HTMLConfig {
            included_selectors: vec![".yes".to_string()],
            excluded_selectors: vec![],
            title_selector: "h1".to_string(),
            save_nearest_id: true,
        };

        let (text, words) = super::parse(html, &config).unwrap();

        assert_eq!(text, "");
    }

    #[test]
    fn random_text_nodes() {
        let html = r#"
            <html>
                <body>
                    <div id="content">
                        <p class="yes">This is a random text node. <span>This is another <i>with italics.</i></span></p>
                        <p>Some more text</p>
                    </div>
                </body>
            </html>"#;

        let config = HTMLConfig {
            included_selectors: vec![".yes".to_string()],
            excluded_selectors: vec![],
            title_selector: "h1".to_string(),
            save_nearest_id: true,
        };

        let (text, words) = super::parse(html, &config).unwrap();

        assert_eq!(
            text,
            "This is a random text node. This is another with italics."
        );
    }

    #[test]
    fn img_alt_text_extraction() {
        let html = r#"
            <html>
                <body>
                    <div id="content">
                        <p class="yes">
                            <span>This is some text.</span>
                            <img src="https://stork-search.net/logo.svg" alt="This is alt text."></img>
                            <span>This is more text.</span>
                        </p>
                        <p>Some more text</p>
                    </div>
                </body>
            </html>"#;

        let config = HTMLConfig {
            included_selectors: vec![".yes".to_string()],
            excluded_selectors: vec![],
            title_selector: "h1".to_string(),
            save_nearest_id: true,
        };

        let (text, words) = super::parse(html, &config).unwrap();

        assert_eq!(
            text,
            "This is some text. This is alt text. This is more text."
        );
    }

    #[test]
    fn title_text_extraction() {
        let html = r#"
            <html>
                <body>
                    <div id="content">
                        <p class="yes">
                            <span>This is the first text.</span>
                            <span title="This is title text.">This is some text.</span>
                            <span>This is more text.</span>
                        </p>
                        <p>Some more text</p>
                    </div>
                </body>
            </html>"#;

        let config = HTMLConfig {
            included_selectors: vec![".yes".to_string()],
            excluded_selectors: vec![],
            title_selector: "h1".to_string(),
            save_nearest_id: true,
        };

        let (text, words) = super::parse(html, &config).unwrap();

        assert_eq!(
            text,
            "This is the first text. This is title text. This is some text. This is more text."
        );
    }

    #[test]
    fn nearest_id() {
        use itertools::Itertools;
        let html = r#"
            <html>
                <body>
                    <main>
                        <p>This is some text.</p>
                        <h1 id="h1">This is title text.</h1>
                        <p>This is text after the h1.</p>
                        <p id="ptag">This is more text after the h1.</p>
                        <p>This is after the h1 and after the p with the id.</p>
                        <aside id="aside">
                            <p>This is text in an aside.</p>
                        </aside>
                    </main>
                </body>
            </html>"#;

        let config = HTMLConfig {
            included_selectors: vec!["main".to_string()],
            excluded_selectors: vec![],
            title_selector: "h1".to_string(),
            save_nearest_id: true,
        };

        let (text, words) = super::parse(html, &config).unwrap();

        #[derive(Debug, PartialEq)]
        struct WordList {
            text: String,
            nearest_id: Option<String>,
        }

        let mut computed: Vec<WordList> = Vec::new();
        for (key, group) in &words.iter().group_by(|w| w.annotation.url_suffix.clone()) {
            computed.push(WordList {
                text: group.map(|aw| aw.word.clone()).collect_vec().join(" "),
                nearest_id: key,
            })
        }

        let expected = vec![
            WordList {
                text: "this is some text".to_string(),
                nearest_id: None,
            },
            WordList {
                text: "this is title text this is text after the h1".to_string(),
                nearest_id: Some("#h1".to_string()),
            },
            WordList {
                text: "this is more text after the h1 this is after the h1 and after the p with the id".to_string(),
                nearest_id: Some("#ptag".to_string()),
            },
            WordList {
                text: "this is text in an aside".to_string(),
                nearest_id: Some("#aside".to_string()),
            },
        ];

        assert_eq!(computed, expected);
    }
}
