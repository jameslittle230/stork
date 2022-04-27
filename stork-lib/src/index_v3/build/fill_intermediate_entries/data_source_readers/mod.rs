use crate::config::{DataSource, Filetype};

use super::{ReadResult, ReaderConfig, WordListGenerationError};

pub mod filepath_data_source_reader;
pub mod url_data_source_reader;

pub fn read_from_data_source(
    reader_config: &ReaderConfig,
) -> Result<ReadResult, WordListGenerationError> {
    match &reader_config.file.source() {
        DataSource::Contents(contents) => Ok(ReadResult {
            buffer: contents.clone(),
            filetype: reader_config
                .file
                .filetype
                .clone()
                .or(Some(Filetype::PlainText)),
            frontmatter_fields: None,
        }),

        DataSource::URL(url) => return url_data_source_reader::read(url, reader_config),
        DataSource::FilePath(path) => filepath_data_source_reader::read(path, reader_config),
    }
    .map(|read_result| read_result.extract_frontmatter(reader_config))
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use pretty_assertions::assert_eq;

    use crate::{
        config::{DataSource, File, Filetype, InputConfig, OutputConfig},
        index_v3::build::fill_intermediate_entries::{ReadResult, ReaderConfig},
    };

    use super::read_from_data_source;

    #[test]
    fn read_from_data_source_extracts_frontmatter() {
        let read_result = read_from_data_source(&ReaderConfig {
            global: InputConfig {
                frontmatter_handling: crate::config::FrontmatterConfig::Parse,
                ..Default::default()
            },
            file: File {
                title: "Input File".to_string(),
                explicit_source: Some(DataSource::Contents(
                    r#"---
key: value
---

# Header

this _is_ the text"#
                        .to_string(),
                )),
                filetype: Some(Filetype::Markdown),
                ..Default::default()
            },
            output: OutputConfig::default(),
        })
        .unwrap();

        assert_eq!(
            read_result,
            ReadResult {
                buffer: "# Header\n\nthis _is_ the text".to_string(),
                filetype: Some(Filetype::Markdown),
                frontmatter_fields: Some(HashMap::from([("key".to_string(), "value".to_string())]))
            }
        );
    }
}
