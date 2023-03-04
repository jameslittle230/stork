use crate::{
    build_config::{Config, DataSource, Filetype},
    Fields,
};

use crate::build_output::document_problem::{AttributedDocumentProblem, DocumentProblem};

pub(crate) mod file;
pub(crate) mod url;

mod frontmatter;

/// After a document is read from its source, we have a ReadContentsOutput.
/// This contains the contents of the document before its markup is parsed,
/// as well as the filetype and the frontmatter, if that can be known and extracted.
pub(crate) struct FileReadValue {
    pub(crate) contents: String,

    /// If the filetype can be read from the data source, the value will be
    /// stored here. When the builder gets to a word list generator, it should
    /// use the filetype here if it's available.
    pub(crate) filetype: Option<Filetype>,

    pub(crate) frontmatter: Option<Fields>, // TODO: Read this field
}

pub(crate) fn read(
    config: &Config,
    file_index: usize,
) -> Result<FileReadValue, AttributedDocumentProblem> {
    let file_config = config.input.files.get(file_index).unwrap();
    let result: Result<(String, Option<Filetype>), DocumentProblem> = match file_config.source() {
        Ok(DataSource::Contents(contents)) => {
            let filetype = file_config.filetype.clone().or(Some(Filetype::PlainText));
            Ok((contents, filetype))
        }

        Ok(DataSource::URL(url)) => url::read(&url).map_err(DocumentProblem::from),
        Ok(DataSource::FilePath(path)) => file::read(&path, config).map_err(DocumentProblem::from),
        Err(_) => Err(DocumentProblem::MultipleContentSources),
    };

    if let Err(problem) = result {
        return Err(AttributedDocumentProblem {
            doc_title: file_config.title.clone(),
            problem,
        });
    }

    // TODO: This unwrap feels weird, but refactoring this to be in a match also
    // feels weird. What's the idiomatic thing to do here?
    let (contents, filetype) = result.unwrap();

    if contents.is_empty() {
        return Err(AttributedDocumentProblem {
            doc_title: file_config.title.clone(),
            problem: DocumentProblem::EmptyWordList,
        });
    }

    // parse frontmatter
    let frontmatter_config = config.get_frontmatter_config_for_file(file_index);
    let (frontmatter, contents) = frontmatter::parse(&frontmatter_config, &contents);

    Ok(FileReadValue {
        contents,
        filetype,
        frontmatter: Some(frontmatter),
    })
}

// TODO: Restore these tests
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
