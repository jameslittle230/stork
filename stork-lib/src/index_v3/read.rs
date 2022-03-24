use super::Index;
use bytes::Bytes;
use compression::prelude::*;
use std::convert::{TryFrom, TryInto};

impl TryFrom<&[u8]> for Index {
    type Error = rmp_serde::decode::Error;
    fn try_from(file: &[u8]) -> Result<Self, Self::Error> {
        let (version_size_bytes, rest) = file.split_at(std::mem::size_of::<u64>());
        let version_size = u64::from_be_bytes(version_size_bytes.try_into().unwrap());
        let version_size: usize = version_size.try_into().unwrap();
        let (version_bytes, rest) = rest.split_at(version_size);

        let (index_size_bytes, rest) = rest.split_at(std::mem::size_of::<u64>());
        let index_size = u64::from_be_bytes(index_size_bytes.try_into().unwrap());
        let index_size: usize = index_size.try_into().unwrap();
        let (index_bytes, _rest) = rest.split_at(index_size);

        if version_bytes == Bytes::from("stork-4").as_ref() {
            rmp_serde::from_read_ref(
                &index_bytes
                    .iter()
                    .cloned()
                    .decode(&mut BZip2Decoder::new())
                    .collect::<Result<Vec<_>, _>>()
                    .unwrap(),
            )
        } else if version_bytes == Bytes::from("stork-3") {
            rmp_serde::from_read_ref(index_bytes)
        } else {
            unreachable!()
        }
    }
}

impl TryFrom<Bytes> for Index {
    type Error = rmp_serde::decode::Error;

    fn try_from(value: Bytes) -> Result<Self, Self::Error> {
        rmp_serde::from_read_ref(value.as_ref())
    }
}
