use std::{
    fs::File,
    io::{stdout, BufWriter, Read, Write},
};

use bytes::Bytes;

use crate::errors::StorkCommandLineError;

pub fn read_stdin_bytes() -> Option<Bytes> {
    use atty::Stream;
    use std::io;

    if atty::isnt(Stream::Stdin) {
        let mut stdin_buffer = Vec::<u8>::new();
        let _read_result = io::stdin().read_to_end(&mut stdin_buffer);
        return Some(Bytes::from(stdin_buffer));
    }

    None
}

pub fn read_bytes_from_path(path: &str) -> Result<Bytes, StorkCommandLineError> {
    if path == "-" {
        return match read_stdin_bytes() {
            Some(stdin) => Ok(stdin),
            None => Err(StorkCommandLineError::InteractiveStdinNotAllowed),
        };
    }

    // TODO: Handle path == "" case
    let pathbuf = std::path::PathBuf::from(path);
    std::fs::read(pathbuf)
        .map(Bytes::from)
        .map_err(|e| StorkCommandLineError::FileReadError(path.to_string(), e))
}

pub fn read_stdin() -> Option<String> {
    read_stdin_bytes().and_then(|bytes| String::from_utf8(Vec::from(bytes.as_ref())).ok())
}

pub fn read_from_path(path: &str) -> Result<String, StorkCommandLineError> {
    match (path, read_stdin()) {
        ("-", Some(stdin)) => Ok(stdin),
        ("-", None) => Err(StorkCommandLineError::InteractiveStdinNotAllowed),
        // handle ("", Some) or ("", None), perhaps
        _ => {
            let pathbuf = std::path::PathBuf::from(path);
            std::fs::read_to_string(pathbuf)
                .map_err(|e| StorkCommandLineError::FileReadError(path.to_string(), e))
        }
    }
}

pub fn write_bytes(path: &str, bytes: &Bytes) -> Result<usize, StorkCommandLineError> {
    let mut writer: Box<dyn Write> = if path == "-" {
        Box::new(stdout())
    } else {
        let file = File::create(path)
            .map_err(|e| StorkCommandLineError::FileCreateError(path.to_string(), e))?;
        Box::new(BufWriter::new(file))
    };

    writer
        .write(bytes.as_ref())
        .map_err(StorkCommandLineError::WriteError)
}
