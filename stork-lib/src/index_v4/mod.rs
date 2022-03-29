use std::mem;

use compression::prelude::*;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use bytes::{BufMut, Bytes, BytesMut};

use crate::{Config, IndexMetadata, IndexParseError, StorkIndex, V3Index};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct V4Index {
    serialization_metadata: SerializationMetadata,

    index_metadata: IndexMetadata,

    #[serde(with = "serde_bytes")]
    index_bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompressionMethod {
    None,
    BZip2,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SerializationMetadata {
    compression_method: CompressionMethod,
}

impl StorkIndex for V4Index {
    fn metadata(&self) -> IndexMetadata {
        self.index_metadata.clone()
    }
}

impl V4Index {
    pub fn into_bytes(&self) -> Bytes {
        let index_bytes = rmp_serde::to_vec(&self).unwrap();
        let index_bytes = Bytes::from(index_bytes);

        let version_bytes = Bytes::from("stork-4");

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

impl TryFrom<Bytes> for V4Index {
    type Error = IndexParseError;

    fn try_from(value: Bytes) -> Result<Self, Self::Error> {
        rmp_serde::from_read_ref(value.as_ref())
            .map_err(|rmp_error| IndexParseError::V3SerdeError(rmp_error.to_string()))
    }
}

impl From<&V4Index> for V3Index {
    fn from(index: &V4Index) -> Self {
        match index.serialization_metadata.compression_method {
            CompressionMethod::None => rmp_serde::from_read_ref(&index.index_bytes).unwrap(),
            CompressionMethod::BZip2 => {
                // let compressed_bytes = index.index_bytes;
                let uncompressed_bytes = &(index.index_bytes.clone())
                    .into_iter()
                    .decode(&mut BZip2Decoder::new())
                    .collect::<Result<Vec<_>, _>>()
                    .unwrap();
                rmp_serde::from_read_ref(&uncompressed_bytes).unwrap()
            }
        }
    }
}

#[cfg(feature = "build-v3")]
impl From<&V4Index> for Bytes {
    fn from(_: &V4Index) -> Self {
        todo!()
    }
}

#[cfg(feature = "build-v3")]
pub struct BuildOutput {
    pub index: V4Index,
    pub warnings: Vec<BuildWarning>,
    pub metadata: IndexMetadata,
}

#[cfg(feature = "build-v3")]
pub enum BuildWarning {}

#[cfg(feature = "build-v3")]
#[derive(Debug, Error)]
pub enum BuildError {
    #[error("{0}")]
    IndexGenerationError(#[from] crate::index_v3::IndexGenerationError), // TODO: BuildError should be an IndexGenerationError
}

#[cfg(feature = "build-v3")]
pub fn build(config: &Config) -> Result<BuildOutput, BuildError> {
    let v3_index_build_result = crate::index_v3::build(config)?;
    let index_bytes = {
        let index_bytes = rmp_serde::to_vec(&v3_index_build_result.index).unwrap();
        match config.output.compression_method {
            CompressionMethod::None => Bytes::from(index_bytes),
            CompressionMethod::BZip2 => Bytes::from(
                (&index_bytes)
                    .into_iter()
                    .cloned()
                    .encode(&mut BZip2Encoder::new(9), Action::Finish)
                    .collect::<Result<Vec<_>, _>>()
                    .unwrap(),
            ),
        }
    };

    let metadata = v3_index_build_result.index.metadata().clone();
    let index = V4Index {
        serialization_metadata: SerializationMetadata {
            compression_method: config.output.compression_method.clone(),
        },
        index_metadata: metadata.clone(),
        index_bytes: index_bytes.to_vec(),
    };

    Ok(BuildOutput {
        index,
        warnings: vec![],
        metadata,
    })
}
