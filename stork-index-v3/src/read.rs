use bytes::Bytes;
use stork_shared::StorkIndex;

use crate::Index;
use std::convert::{TryFrom, TryInto};

impl TryFrom<&[u8]> for Index {
    type Error = rmp_serde::decode::Error;
    fn try_from(file: &[u8]) -> Result<Self, Self::Error> {
        let (version_size_bytes, rest) = file.split_at(std::mem::size_of::<u64>());
        let version_size = u64::from_be_bytes(version_size_bytes.try_into().unwrap());
        let version_size: usize = version_size.try_into().unwrap();
        let (_version_bytes, rest) = rest.split_at(version_size);

        let (index_size_bytes, rest) = rest.split_at(std::mem::size_of::<u64>());
        let index_size = u64::from_be_bytes(index_size_bytes.try_into().unwrap());
        let index_size: usize = index_size.try_into().unwrap();
        let (index_bytes, _rest) = rest.split_at(index_size);

        rmp_serde::from_read_ref(index_bytes)
    }
}

impl TryFrom<Bytes> for Index {
    type Error = rmp_serde::decode::Error;

    fn try_from(value: Bytes) -> Result<Self, Self::Error> {
        rmp_serde::from_read_ref(value.as_ref())
    }
}

impl StorkIndex for Index {}
