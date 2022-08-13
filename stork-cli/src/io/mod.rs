use std::{
    fs::File,
    io::{stdout, BufWriter, Read, Write},
};

use bytes::Bytes;

use crate::errors::CommandLineError;

fn read_stdin_bytes() -> Option<Bytes> {
    use atty::Stream;
    use std::io;

    if atty::isnt(Stream::Stdin) {
        let mut stdin_buffer = Vec::<u8>::new();
        let _read_result = io::stdin().read_to_end(&mut stdin_buffer);
        return Some(Bytes::from(stdin_buffer));
    }

    None
}

pub(crate) fn read(path: &str) -> Result<Bytes, CommandLineError> {
    match (path, read_stdin_bytes()) {
        ("-", Some(stdin)) => Ok(stdin),
        ("-", None) => Err(CommandLineError::InteractiveStdinNotAllowed),
        // TODO: handle ("", Some) or ("", None), perhaps
        _ => {
            let pathbuf = std::path::PathBuf::from(path);
            std::fs::read(&pathbuf)
                .map(std::convert::Into::into)
                .map_err(|e| CommandLineError::FileReadError(path.to_string(), e))
        }
    }
}

pub(crate) fn write(path: &str, bytes: &Bytes) -> Result<usize, CommandLineError> {
    let mut writer: Box<dyn Write> = if path == "-" {
        Box::new(stdout())
    } else {
        let file = File::create(path)
            .map_err(|e| CommandLineError::FileCreateError(path.to_string(), e))?;
        Box::new(BufWriter::new(file))
    };

    writer
        .write(bytes.as_ref())
        .map_err(CommandLineError::WriteError)
}
