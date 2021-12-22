use std::mem;

use bytes::{BufMut, Bytes, BytesMut};

use crate::Index;

impl From<&Index> for Bytes {
    fn from(value: &Index) -> Self {
        let index_bytes = rmp_serde::to_vec(&value).unwrap();
        let index_bytes = Bytes::from(index_bytes);
        let version_bytes = Bytes::from("stork-3");

        let mut buf = BytesMut::with_capacity(
            index_bytes.len() + version_bytes.len() + 2 * mem::size_of::<u64>(),
        );
        buf.put_u64(version_bytes.len() as u64);

        buf.put(version_bytes);
        buf.put_u64(index_bytes.len() as u64);
        buf.put(index_bytes);

        buf.freeze()
    }
}
