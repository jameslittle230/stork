use pulldown_cmark::{html as PulldownHtml, Parser as PulldownParser};

use crate::build_config::HTMLConfig;

use super::annotated_word::AnnotatedWord;

pub(crate) fn parse(
    contents: &str,
    html_config: &HTMLConfig,
) -> Result<(String, Vec<AnnotatedWord>), super::html::HtmlParseError> {
    let parser = PulldownParser::new(contents);
    let mut html_output = String::new();
    PulldownHtml::push_html(&mut html_output, parser);

    let html_string = format!("<html><body><main>{html_output}</main></body></html>");

    super::html::parse(&html_string, html_config)
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::build_config::HTMLConfig;

    #[test]
    fn markdown_text_extraction() {
        let md = r#"
# This is a title

Stork should recognize this text

- This content should be indexed.
- This is another paragraph with **_inline text_ formatting**.
- [This is a link.](https://example.com)

Goodbye!"#;

        let config = HTMLConfig {
            included_selectors: vec!["main".to_string()],
            excluded_selectors: vec![],
            title_selector: "h1".to_string(),
            save_nearest_id: true,
        };

        let (text, words) = super::parse(md, &config).unwrap();

        assert_eq!(
            text,
            "This is a title Stork should recognize this text This content should be indexed. This is another paragraph with inline text formatting . This is a link. Goodbye!"
        );
    }

    #[test]
    fn issue_290_space_after_numeric_list() {
        let md = r#"1. 

something below, there is a space immediately after the 1. above  "#;

        let config = HTMLConfig {
            included_selectors: vec!["main".to_string()],
            excluded_selectors: vec![],
            title_selector: "h1".to_string(),
            save_nearest_id: true,
        };

        let (text, words) = super::parse(md, &config).unwrap();

        assert_eq!(
            text,
            "something below, there is a space immediately after the 1. above"
        );
    }

    #[test]
    fn issue_290_space_after_bullet_list() {
        let md = r#"* 

something below, there is a space immediately after the star above  "#;

        let config = HTMLConfig {
            included_selectors: vec!["main".to_string()],
            excluded_selectors: vec![],
            title_selector: "h1".to_string(),
            save_nearest_id: true,
        };

        let (text, words) = super::parse(md, &config).unwrap();

        assert_eq!(
            text,
            "something below, there is a space immediately after the star above"
        );
    }
}
