use super::{ReadResult, ReaderConfig, WordListGenerationError};
use crate::config::{DataSource, Filetype};

pub mod filepath_data_source_reader;
pub mod url_data_source_reader;

pub fn read_from_data_source(
    reader_config: &ReaderConfig,
) -> Result<ReadResult, WordListGenerationError> {
    match &reader_config.file.source() {
        DataSource::Contents(contents) => Ok(ReadResult {
            buffer: contents.to_owned(),
            filetype: reader_config
                .file
                .filetype
                .clone()
                .or(Some(Filetype::PlainText)),
            frontmatter_fields: None,
        }),

        DataSource::URL(url) => url_data_source_reader::read(&url, &reader_config),
        DataSource::FilePath(path) => filepath_data_source_reader::read(&path, &reader_config),
    }
    .map(|read_result| read_result.extract_frontmatter(reader_config))
}
