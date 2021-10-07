use bytes::{BufMut, Bytes, BytesMut};

use crate::Index;

impl From<&Index> for Bytes {
    fn from(value: &Index) -> Self {
        let index_bytes = rmp_serde::to_vec(&value).unwrap();
        let index_bytes = Bytes::from(index_bytes);

        let version_bytes = Bytes::from("stork-v3");

        let mut buf = BytesMut::with_capacity(index_bytes.len() + version_bytes.len());
        buf.put(Bytes::from("stork-v3".as_bytes()));
        buf.put(index_bytes);

        buf.freeze()
    }
}
