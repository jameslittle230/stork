use crate::LatestVersion::structs::AnnotatedWordList;

use super::{html_word_list_generator, ReadResult, ReaderConfig, WordListGenerationError};

pub fn generate(
    config: &ReaderConfig,
    read_result: &ReadResult,
) -> Result<AnnotatedWordList, WordListGenerationError> {
    let html_string = format!(
        "<html><body><main>{}</main></body></html>",
        markdown::to_html(&read_result.buffer)
    );

    let html_read_result = ReadResult {
        buffer: html_string,
        filetype: read_result.filetype.clone(),
        frontmatter_fields: None,
    };
    html_word_list_generator::generate(config, &html_read_result)
}

#[cfg(test)]
mod tests {
    use crate::{
        config::{File, Filetype, InputConfig, OutputConfig},
        LatestVersion::builder::fill_intermediate_entries::{ReadResult, ReaderConfig},
    };

    use super::generate;

    #[test]
    fn test_markdown() {
        let expected = "This is a title Stork should recognize this text This content should be indexed. This is another paragraph with inline text formatting . This is a link. Goodbye!";
        let computed: String = generate(
            &ReaderConfig {
                global: InputConfig::default(),
                file: File::default(),
                output: OutputConfig::default(),
            },
            &ReadResult {
                buffer: r#"
# This is a title

Stork should recognize this text

- This content should be indexed.
- This is another paragraph with **_inline text_formatting**.
- [This is a link.](https://example.com)

Goodbye!"#
                    .to_string(),
                filetype: Some(Filetype::Markdown),
                frontmatter_fields: None,
            },
        )
        .ok()
        .unwrap()
        .word_list
        .iter()
        .map(|aw| aw.word.clone().trim().to_string())
        .collect::<Vec<String>>()
        .join(" ");

        assert_eq!(expected, computed);
    }
}
