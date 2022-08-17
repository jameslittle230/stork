use bytes::Bytes;

use std::convert::TryFrom;

use crate::envelope;
// use crate::index_v2::Index as V2IndexType;
// use crate::index_v3::Index as V3IndexType;
use crate::index_v4::IndexDiskRepresentation as V4IndexType;

pub mod errors;

/// An index-version-agnostic wrapper type to represent a usable search index.
pub struct ParsedIndex {
    pub(crate) value: IndexType,
}

/// A private enum
pub(crate) enum IndexType {
    // V2Index(V2IndexType),
    // V3Index(V3IndexType),
    V4Index(V4IndexType),
}

pub(super) fn parse(bytes: Bytes) -> Result<ParsedIndex, errors::IndexParseError> {
    let envelope = envelope::Envelope::try_from(bytes)?;

    match envelope.prefix {
        envelope::Prefix::StorkV2 => {
            // if !cfg!(feature = "search-v2") {
            return Err(errors::IndexParseError::NotCompiledWithFeature(
                "search-v2".to_string(),
            ));
            // } else {
            //     return V2IndexType::try_from(envelope.bytes.first().unwrap())
            //         .map_err(|e| IndexParseError::V2IndexDeserializeError(e.to_string()))
            //         .map(|index| ParsedIndex {
            //             value: IndexType::V2Index(index),
            //         });
            // }
        }

        envelope::Prefix::StorkV3 => {
            // if !cfg!(feature = "search-v2") {
            return Err(errors::IndexParseError::NotCompiledWithFeature(
                "search-v3".to_string(),
            ));
            // } else {
            //     return V3IndexType::try_from(envelope.bytes.first().unwrap())
            //         .map_err(|e| IndexParseError::V3IndexDeserializeError(e))
            //         .map(|index| ParsedIndex {
            //             value: IndexType::V3Index(index),
            //         });
            // }
        }
        envelope::Prefix::StorkV4 => {
            // Index v5: Put this behind a feature conditional
            return V4IndexType::try_from(envelope.bytes.first().unwrap())
                .map_err(|e| errors::IndexParseError::V4IndexDeserializeError(e))
                .map(|index| ParsedIndex {
                    value: IndexType::V4Index(index),
                });
        }
    }
}

pub(super) fn add_sidecar_bytes_to_index(
    index: &mut ParsedIndex,
    bytes: Bytes,
) -> Result<(), errors::IndexParseError> {
    todo!()
}
