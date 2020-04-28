use crate::IndexFromFile;
use std::convert::TryInto;
use std::fmt;
use std::error::Error;

#[derive(Debug)]
pub struct IndexParseError {}

impl Error for IndexParseError {}

impl fmt::Display for IndexParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Could not parse index file.")
    }
}

pub fn get_index_version(index: &IndexFromFile) -> Result<String, IndexParseError> {
    let (version_size_bytes, rest) = index.split_at(std::mem::size_of::<u64>());
    if let Ok(byte_array) = version_size_bytes.try_into() {
        let version_size = u64::from_be_bytes(byte_array);
        let (version_bytes, _rest) = rest.split_at(version_size as usize);
        String::from_utf8(version_bytes.to_vec()).map_err(|_err| IndexParseError{})
    } else {
        Err(IndexParseError{})
    }
}
