use pulldown_cmark::{html as PulldownHtml, Parser as PulldownParser};

use super::annotated_word::AnnotatedWord;

pub(crate) fn generate(
    config: &crate::config::Config,
    file_index: usize,
    contents: &str,
) -> Result<(String, Vec<AnnotatedWord>), super::html::HtmlParseError> {
    let parser = PulldownParser::new(contents);
    let mut html_output = String::new();
    PulldownHtml::push_html(&mut html_output, parser);

    let html_string = format!("<html><body><main>{html_output}</main></body></html>");

    super::html::generate(config, file_index, &html_string)
}

// TODO: Restore tests
// #[cfg(test)]
// mod tests {

//     use pretty_assertions::assert_eq;

//     use crate::{
//         build::fill_intermediate_entries::{ReadResult, ReaderConfig},
//         config::{File, Filetype, InputConfig, OutputConfig},
//     };

//     use super::generate;

//     fn assert_markdown_content(word_list: &str, markdown_content: &str) {
//         let computed: String = generate(
//             &ReaderConfig {
//                 global: InputConfig {
//                     frontmatter_handling: crate::config::FrontmatterConfig::Omit,
//                     ..InputConfig::default()
//                 },
//                 file: File::default(),
//                 output: OutputConfig::default(),
//             },
//             &ReadResult {
//                 buffer: markdown_content.to_string(),
//                 filetype: Some(Filetype::Markdown),
//                 frontmatter_fields: None,
//             },
//         )
//         .unwrap()
//         .word_list
//         .iter()
//         .map(|aw| aw.word.clone().trim().to_string())
//         .collect::<Vec<String>>()
//         .join(" ");

//         assert_eq!(word_list, computed);
//     }

//     #[test]
//     fn test_markdown() {
//         assert_markdown_content(
//             "This is a title Stork should recognize this text This content should be indexed. This is another paragraph with inline text formatting . This is a link. Goodbye!",
// r#"
// # This is a title

// Stork should recognize this text

// - This content should be indexed.
// - This is another paragraph with **_inline text_ formatting**.
// - [This is a link.](https://example.com)

// Goodbye!"#
// );
//     }

//     mod issue_290 {

//         use super::assert_markdown_content;

//         #[test]
//         #[rustfmt::skip]
//         fn space_after_numeric_list() {
//             assert_markdown_content(
//                 "something below, there is a space immediately after the 1. above",
//                 r#"1.

// something below, there is a space immediately after the 1. above  "#,
//             );
//         }

//         #[test]
//         #[rustfmt::skip]
//         fn space_after_bullet_list() {
//             assert_markdown_content(
//                 "something below, there is a space immediately after the star above",
//     r#"*

// something below, there is a space immediately after the star above  "#,
//             );
//         }
//     }
// }