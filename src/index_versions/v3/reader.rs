use super::structs::Index;
use crate::common::IndexFromFile;
use std::convert::{TryFrom, TryInto};

impl TryFrom<&IndexFromFile> for Index {
    type Error = rmp_serde::decode::Error;

    #[allow(clippy::cast_possible_truncation)]
    fn try_from(file: &IndexFromFile) -> Result<Self, Self::Error> {
        let (version_size_bytes, rest) = file.split_at(std::mem::size_of::<u64>());
        let version_size = u64::from_be_bytes(version_size_bytes.try_into().unwrap());
        let (_version_bytes, rest) = rest.split_at(version_size as usize);

        let (index_size_bytes, rest) = rest.split_at(std::mem::size_of::<u64>());
        let index_size = u64::from_be_bytes(index_size_bytes.try_into().unwrap());
        let (index_bytes, _rest) = rest.split_at(index_size as usize);

        rmp_serde::from_read_ref(index_bytes)
    }
}
