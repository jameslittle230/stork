use thiserror::Error;

use crate::envelope;

/// An error thrown when Stork has failed to parse an index from some bytes,
/// or failed to append a secondary chunk onto an existing ParsedIndex.
#[derive(Error, Debug)]
pub enum IndexParseError {
    #[error("")]
    UnknownFile(#[from] envelope::EnvelopeDecodeError),

    #[error("")]
    V2IndexDeserializeError(String),

    #[cfg(feature = "search-v3")]
    #[error("")]
    V3IndexDeserializeError(rmp_serde::decode::Error),

    #[error("")]
    V4IndexDeserializeError(rmp_serde::decode::Error),

    #[error("")]
    NotCompiledWithFeature(String),
}
