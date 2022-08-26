use thiserror::Error;

use crate::build_config;

use std::io::Read;
use std::{fs, io, path};

#[derive(Debug, Clone, PartialEq, Error)]
pub(crate) enum FileReadError {
    #[error("")]
    FileNotFound(path::PathBuf),
}

pub(crate) fn read(
    path: &str,
    config: &crate::build_config::Config,
) -> Result<(String, Option<build_config::Filetype>), FileReadError> {
    let base_directory_path = path::Path::new(&config.input.base_directory);
    let full_pathname = base_directory_path.join(&path);

    let file = fs::File::open(&full_pathname)
        .map_err(|_| FileReadError::FileNotFound(full_pathname.clone()))?;

    let mut buf_reader = io::BufReader::new(file);
    let mut buffer = String::new();
    let _bytes_read = buf_reader.read_to_string(&mut buffer);

    let filetype_from_extension = get_filetype_from_path(&full_pathname);

    Ok((buffer, filetype_from_extension))
}

fn get_filetype_from_path(path: &path::Path) -> Option<build_config::Filetype> {
    let ext_str = path.extension()?.to_str()?;
    match String::from(ext_str).to_ascii_lowercase().as_ref() {
        "html" | "htm" => Some(build_config::Filetype::HTML),
        "srt" => Some(build_config::Filetype::SRTSubtitle),
        "txt" => Some(build_config::Filetype::PlainText),
        "markdown" | "mdown" | "md" => Some(build_config::Filetype::Markdown),
        _ => None,
    }
}
