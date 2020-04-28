use crate::IndexFromFile;
use std::convert::TryInto;
use std::error::Error;
use std::fmt;

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
        String::from_utf8(version_bytes.to_vec()).map_err(|_err| IndexParseError {})
    } else {
        Err(IndexParseError {})
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::{BufReader, Read};

    #[test]
    fn can_get_version_of_0_5_3_index() {
        let file = fs::File::open("./test-assets/federalist-min.st").unwrap();
        let mut buf_reader = BufReader::new(file);
        let mut index_bytes: Vec<u8> = Vec::new();
        let _bytes_read = buf_reader.read_to_end(&mut index_bytes);
        let result = get_index_version(index_bytes.as_slice());
        assert_eq!("stork-2", result.unwrap());
    }
}
