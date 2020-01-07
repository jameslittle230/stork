use std::collections::HashMap;
use std::convert::TryInto;

use crate::models::{StorkEntry, StorkResultOrAlias};

pub fn get_index_version(index: &[u8]) -> String {
    let (version_size_bytes, rest) = index.split_at(std::mem::size_of::<u64>());
    let version_size = u64::from_be_bytes(version_size_bytes.try_into().unwrap());
    let (version_bytes, _rest) = rest.split_at(version_size as usize);
    let version = String::from_utf8(version_bytes.to_vec()).unwrap();
    return version;
}

pub fn get_index_entries(index: &[u8]) -> Vec<StorkEntry> {
    let (version_size_bytes, rest) = index.split_at(std::mem::size_of::<u64>());
    let version_size = u64::from_be_bytes(version_size_bytes.try_into().unwrap());
    let (_version_bytes, rest) = rest.split_at(version_size as usize);

    let (entries_size_bytes, rest) = rest.split_at(std::mem::size_of::<u64>());
    let entries_size = u64::from_be_bytes(entries_size_bytes.try_into().unwrap());
    let (entries_bytes, _rest) = rest.split_at(entries_size as usize);
    return bincode::deserialize(entries_bytes).unwrap();
}

pub fn get_index_results(index: &[u8]) -> HashMap<String, Vec<StorkResultOrAlias>> {
    let (version_size_bytes, rest) = index.split_at(std::mem::size_of::<u64>());
    let version_size = u64::from_be_bytes(version_size_bytes.try_into().unwrap());
    let (_version_bytes, rest) = rest.split_at(version_size as usize);

    let (entries_size_bytes, rest) = rest.split_at(std::mem::size_of::<u64>());
    let entries_size = u64::from_be_bytes(entries_size_bytes.try_into().unwrap());
    let (_entries_bytes, rest) = rest.split_at(entries_size as usize);

    let (results_size_bytes, rest) = rest.split_at(std::mem::size_of::<u64>());
    let results_size = u64::from_be_bytes(results_size_bytes.try_into().unwrap());
    let (results_bytes, _rest) = rest.split_at(results_size as usize);
    return bincode::deserialize(results_bytes).unwrap();
}
