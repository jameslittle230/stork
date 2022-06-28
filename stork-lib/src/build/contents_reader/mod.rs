use crate::{
    config::{DataSource, Filetype},
    Fields,
};

use super::errors::{AttributedDocumentReadError, DocumentReadError};

pub mod filepath_data_source_reader;
mod frontmatter;
pub mod url_data_source_reader;

/**
 * After a document is read from its source, we have a ReadContentsOutput.
 * This contains the contents of the document before its markup is parsed,
 * as well as the filetype and the frontmatter, if that can be known and extracted.
 */
pub(crate) struct ReadContentsOutput {
    pub(crate) contents: String,

    /// If the filetype can be read from the data source, the value will be
    /// stored here. When the builder gets to a word list generator, it should
    /// use the filetype here if it's available.
    pub(crate) filetype: Option<Filetype>,

    pub(crate) frontmatter: Option<Fields>,
}

pub(crate) fn read_contents(
    document: &crate::config::File,
    config: &crate::config::Config,
) -> Result<ReadContentsOutput, AttributedDocumentReadError> {
    let result: Result<(String, Option<Filetype>), DocumentReadError> = match document.source() {
        DataSource::Contents(contents) => {
            let contents = contents.to_string();
            let filetype = document.filetype.clone().or(Some(Filetype::PlainText));
            Ok((contents, filetype))
        }

        DataSource::URL(url) => url_data_source_reader::read(&url),
        DataSource::FilePath(path) => filepath_data_source_reader::read(&path, config),
    };

    if let Err(document_read_error) = result {
        return Err(AttributedDocumentReadError {
            read_error: document_read_error,
            document: document.clone(),
        });
    }

    // TODO: This unwrap feels weird, but refactoring this to be in a match also
    // feels weird. What's the idiomatic thing to do here?
    let (contents, filetype) = result.unwrap();

    if contents.is_empty() {
        return Err(AttributedDocumentReadError {
            read_error: DocumentReadError::EmptyWordList,
            document: document.clone(),
        });
    }

    // parse frontmatter
    let frontmatter_handling = document
        .frontmatter_handling_override
        .clone()
        .unwrap_or(config.input.frontmatter_handling.clone());

    let (frontmatter, contents) = frontmatter::parse(&frontmatter_handling, &contents);

    Ok(ReadContentsOutput {
        contents,
        filetype,
        frontmatter: Some(frontmatter),
    })
}

// #[cfg(test)]
// mod tests {
//     use std::collections::HashMap;

//     use pretty_assertions::assert_eq;

//     use crate::{
//         build::fill_intermediate_entries::{ReadResult, ReaderConfig},
//         config::{DataSource, File, Filetype, InputConfig, OutputConfig},
//     };

//     #[test]
//     fn read_from_data_source_extracts_frontmatter() {
//         let read_result = read_from_data_source(&ReaderConfig {
//             global: InputConfig {
//                 frontmatter_handling: crate::config::FrontmatterConfig::Parse,
//                 ..InputConfig::default()
//             },
//             file: File {
//                 title: "Input File".to_string(),
//                 explicit_source: Some(DataSource::Contents(
//                     r#"---
// key: value
// ---

// # Header

// this _is_ the text"#
//                         .to_string(),
//                 )),
//                 filetype: Some(Filetype::Markdown),
//                 ..File::default()
//             },
//             output: OutputConfig::default(),
//         })
//         .unwrap();

//         assert_eq!(
//             read_result,
//             ReadResult {
//                 buffer: "# Header\n\nthis _is_ the text".to_string(),
//                 filetype: Some(Filetype::Markdown),
//                 frontmatter_fields: Some(HashMap::from([("key".to_string(), "value".to_string())]))
//             }
//         );
//     }
// }
