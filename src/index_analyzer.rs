use crate::IndexFromFile;
use std::convert::TryInto;

pub fn get_index_version(index: &IndexFromFile) -> String {
    let (version_size_bytes, rest) = index.split_at(std::mem::size_of::<u64>());
    let version_size = u64::from_be_bytes(version_size_bytes.try_into().unwrap());
    let (version_bytes, _rest) = rest.split_at(version_size as usize);
    String::from_utf8(version_bytes.to_vec()).unwrap()
}