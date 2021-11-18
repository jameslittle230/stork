use super::structs::Index;
use std::{error::Error, fmt};

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
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::<u8>::new();

        let write_version = super::VERSION_STRING.as_bytes();
        let index_bytes = rmp_serde::to_vec(self).unwrap();

        let byte_vectors_to_write = [write_version, index_bytes.as_slice()];

        for vec in &byte_vectors_to_write {
            bytes.extend_from_slice(&(vec.len() as u64).to_be_bytes());
            bytes.extend_from_slice(vec);
        }

        bytes
    }
}
