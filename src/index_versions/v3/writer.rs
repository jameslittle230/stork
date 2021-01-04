use super::structs::Index;
use crate::config::Config;
use std::io::{BufWriter, Write};
use std::{error::Error, fmt, fs::File};

#[derive(Debug)]
pub enum WriteError {
    FileCreateError(String),
}

impl Error for WriteError {}

impl fmt::Display for WriteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let desc: String = match self {
            WriteError::FileCreateError(filename) => {
                format!("Could not write to file {}!", filename)
            }
        };

        write!(f, "{}", desc)
    }
}

impl Index {
    pub fn write(&self, config: &Config) -> Result<usize, WriteError> {
        let config = &config.output;
        let file = File::create(&config.filename)
            .map_err(|_| WriteError::FileCreateError(config.filename.clone()))?;
        let mut bufwriter = BufWriter::new(file);

        if config.debug {
            Ok(self.write_debug(&mut bufwriter))
        } else {
            Ok(self.write_to_buffer(&mut bufwriter))
        }
    }

    pub fn write_to_buffer(&self, buffer: &mut dyn Write) -> usize {
        let mut bytes_written: usize = 0;
        let write_version = super::VERSION_STRING.as_bytes();

        let index_bytes = rmp_serde::to_vec(self).unwrap();

        let byte_vectors_to_write = [write_version, index_bytes.as_slice()];

        for vec in byte_vectors_to_write.iter() {
            bytes_written += buffer.write(&(vec.len() as u64).to_be_bytes()).unwrap();
            bytes_written += buffer.write(vec).unwrap();
        }

        bytes_written
    }

    fn write_debug(&self, bufwriter: &mut dyn Write) -> usize {
        let index_serialized = serde_json::to_string_pretty(self).unwrap();
        let write_version = super::VERSION_STRING.as_bytes();

        let byte_vectors_to_write = [write_version, index_serialized.as_bytes()];

        for vec in byte_vectors_to_write.iter() {
            let _ = bufwriter.write(vec.len().to_string().as_bytes());
            let _ = bufwriter.write(b"\n");
            let _ = bufwriter.write(vec);
            let _ = bufwriter.write(b"\n\n");
        }

        // Return zero bytes written so that the frontend can alert the user
        // when they write an index in debug mode
        0
    }
}
