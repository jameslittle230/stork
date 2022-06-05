use crate::{build::errors::DocumentReadError, config::Filetype};

use std::{
    fs::File,
    io::{BufReader, Read},
    path::Path,
};

pub(crate) fn read(
    path: &str,
    config: &crate::config::Config,
) -> Result<(String, Option<Filetype>), DocumentReadError> {
    let base_directory_path = Path::new(&config.input.base_directory);
    let full_pathname = base_directory_path.join(&path);

    let file = File::open(&full_pathname)
        .map_err(|_| DocumentReadError::FileNotFound(full_pathname.clone()))?;

    let mut buf_reader = BufReader::new(file);
    let mut buffer = String::new();
    let _bytes_read = buf_reader.read_to_string(&mut buffer);

    let filetype_from_extension = get_filetype_from_path(&full_pathname);

    Ok((buffer, filetype_from_extension))
}

fn get_filetype_from_path(path: &Path) -> Option<Filetype> {
    let ext_str = path.extension()?.to_str()?;
    match String::from(ext_str).to_ascii_lowercase().as_ref() {
        "html" | "htm" => Some(Filetype::HTML),
        "srt" => Some(Filetype::SRTSubtitle),
        "txt" => Some(Filetype::PlainText),
        "markdown" | "mdown" | "md" => Some(Filetype::Markdown),
        _ => None,
    }
}
