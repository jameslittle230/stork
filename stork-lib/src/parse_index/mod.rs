use bytes::Bytes;
use wasm_bindgen::prelude::wasm_bindgen;

use std::convert::TryFrom;

use crate::{envelope, index_v4};

pub mod errors;

#[wasm_bindgen]
pub struct IndexStatistics {
    // TODO: Implement
}

/// An index-version-agnostic wrapper type to represent a usable search index.
pub struct ParsedIndex {
    pub(crate) value: IndexType,
}

impl ParsedIndex {
    pub fn stats(&self) -> IndexStatistics {
        IndexStatistics {}
    }
}

pub(crate) enum IndexType {
    V4Index(index_v4::Index),
}

pub(super) fn parse(bytes: Bytes) -> Result<ParsedIndex, errors::IndexParseError> {
    let envelope = envelope::Envelope::try_from(bytes)?;

    match envelope.prefix {
        envelope::Prefix::StorkV4Root => {
            let root = index_v4::RootChunk::try_from(envelope.bytes)?;
            let index = index_v4::Index::from(root);
            Ok(ParsedIndex {
                value: IndexType::V4Index(index),
            })
        }

        envelope::Prefix::StorkV4Part => Err(errors::IndexParseError::TriedParsingChunkAsRoot),
    }
}

pub(super) fn add_sidecar_bytes_to_index(
    index: &mut ParsedIndex,
    bytes: Bytes,
) -> Result<(), errors::IndexParseError> {
    let envelope = envelope::Envelope::try_from(bytes)?;
    let chunk = index_v4::Chunk::try_from(envelope.bytes)?;

    match (envelope.prefix, &mut index.value) {
        (envelope::Prefix::StorkV4Part, IndexType::V4Index(v4_index)) => {
            v4_index.add_chunk(&chunk);
            Ok(())
        }

        (envelope::Prefix::StorkV4Root, _) => Err(errors::IndexParseError::TriedParsingChunkAsRoot),
    }
}
