use super::{frontmatter::parse_frontmatter, word_list_generators::WordListGenerationError};
use super::{word_list_generators::returns_word_list_generator, DocumentError};
use super::{IndexGenerationError, IntermediateEntry};
use crate::config::{Config, DataSource, Filetype, StemmingConfig};
use mime::Mime;
use rust_stemmers::Algorithm;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

pub(super) fn fill_intermediate_entries(
    config: &Config,
    intermediate_entries: &mut Vec<IntermediateEntry>,
    document_errors: &mut Vec<DocumentError>,
) -> Result<(), IndexGenerationError> {
    if config.input.files.is_empty() {
        return Err(IndexGenerationError::NoFilesSpecified);
    }

    let base_directory = Path::new(&config.input.base_directory);

    for stork_file in config.input.files.iter() {
        let mut per_file_input_config = config.input.clone();
        let mut file_mime: Option<Mime> = Option::None;
        per_file_input_config.files = vec![];

        let buffer: String = match &stork_file.source {
            DataSource::Contents(contents) => contents.to_string(),

            DataSource::FilePath(path_string) => {
                let full_pathname = &base_directory.join(&path_string);
                let file = match File::open(&full_pathname) {
                    Ok(file) => file,
                    Err(_e) => {
                        document_errors.push(DocumentError {
                            file: stork_file.clone(),
                            word_list_generation_error: WordListGenerationError::FileNotFound,
                        });
                        continue;
                    }
                };
                let mut buf_reader = BufReader::new(file);
                let mut buffer = String::new();
                let _bytes_read = buf_reader.read_to_string(&mut buffer);
                buffer
            }

            DataSource::URL(url) => {
                let mut cont_with_parsed_mime = || -> Result<String, WordListGenerationError> {
                    let mut resp = reqwest::blocking::get(url)
                        .map_err(|_| WordListGenerationError::WebPageNotFetched)?;
                    let mime_type: Mime = resp
                        .headers()
                        .get(reqwest::header::CONTENT_TYPE)
                        .ok_or_else(|| WordListGenerationError::UnknownContentType)?
                        .to_str()
                        .map_err(|_| WordListGenerationError::UnknownContentType)?
                        .parse()
                        .map_err(|_| WordListGenerationError::UnknownContentType)?;

                    let mut buffer = String::new();
                    let _bytes_read = resp.read_to_string(&mut buffer);

                    file_mime = Some(mime_type);
                    Ok(buffer)
                };

                match cont_with_parsed_mime() {
                    Ok(buffer) => buffer,
                    Err(e) => {
                        document_errors.push(DocumentError {
                            file: stork_file.clone(),
                            word_list_generation_error: e,
                        });
                        continue;
                    }
                }
            }
        };

        let current_stem_config = stork_file
            .stemming_override
            .clone()
            .unwrap_or_else(|| config.input.stemming.clone());

        let stem_algorithm: Option<Algorithm> = match current_stem_config {
            StemmingConfig::Language(alg) => Some(alg.to_owned()),
            StemmingConfig::None => None,
        };

        per_file_input_config.html_selector = vec![
            &stork_file.html_selector_override,
            &config.input.html_selector,
        ]
        .into_iter()
        .filter_map(|option| option.to_owned())
        .collect::<Vec<String>>()
        .first()
        .map(|s| s.to_owned());

        per_file_input_config.frontmatter_handling = stork_file
            .frontmatter_handling_override
            .clone()
            .unwrap_or(per_file_input_config.frontmatter_handling);

        let (frontmatter_fields, buffer) = parse_frontmatter(&per_file_input_config, &buffer);

        let filetype_from_mime = file_mime
            .map(|mime| match (mime.type_(), mime.subtype()) {
                (mime::TEXT, mime::PLAIN) => Some(Filetype::PlainText),
                (mime::TEXT, mime::HTML) => Some(Filetype::HTML),
                _ => None,
            })
            .flatten();

        let get_filetype_from_extension = |path_string: &str| {
            let path = Path::new(&path_string);
            let ext_str = path.extension()?.to_str()?;
            match String::from(ext_str).to_ascii_lowercase().as_ref() {
                "html" => Some(Filetype::HTML),
                "htm" => Some(Filetype::HTML),
                "srt" => Some(Filetype::SRTSubtitle),
                "txt" => Some(Filetype::PlainText),
                "md" => Some(Filetype::Markdown),
                "mdown" => Some(Filetype::Markdown),
                "markdown" => Some(Filetype::Markdown),
                _ => None,
            }
        };

        let filetype_from_extension = {
            match &stork_file.source {
                DataSource::FilePath(path_string) => get_filetype_from_extension(&path_string),
                _ => None,
            }
        };

        let user_defined_filetype = stork_file.filetype.clone();

        let ordered_filetype_choices = vec![
            filetype_from_mime,
            filetype_from_extension,
            user_defined_filetype,
        ]
        .into_iter()
        .filter_map(|opt| opt)
        .collect::<Vec<Filetype>>();

        let filetype = match ordered_filetype_choices.last() {
            Some(filetype) => filetype,
            None => {
                document_errors.push(DocumentError {
                    file: stork_file.clone(),
                    word_list_generation_error: WordListGenerationError::CannotDetermineFiletype,
                });
                continue;
            }
        };

        let word_list_result = returns_word_list_generator(&filetype)
            .create_word_list(&per_file_input_config, buffer.as_str());

        match word_list_result {
            Ok(contents) => {
                let mut fields = stork_file.fields.clone();
                fields.extend(frontmatter_fields.into_iter());

                let entry = IntermediateEntry {
                    contents,
                    stem_algorithm,
                    title: stork_file.title.clone(),
                    url: stork_file.url.clone(),
                    fields,
                };

                intermediate_entries.push(entry);
            }
            Err(error) => document_errors.push(DocumentError {
                file: stork_file.clone(),
                word_list_generation_error: error,
            }),
        }
    }

    Ok(())
}
