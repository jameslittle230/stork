use crate::envelope::{self, EnvelopeDecodeError};

/// An error thrown when Stork has failed to parse an index from some bytes,
/// or failed to append a secondary chunk onto an existing ParsedIndex.
#[cfg_attr(feature = "build", derive(Debug))]
pub enum IndexParseError {
    UnknownFile(envelope::EnvelopeDecodeError),
    V4IndexDeserializeError,
    TriedParsingChunkAsRoot,
    NotCompiledWithFeature(String),
}

impl From<EnvelopeDecodeError> for IndexParseError {
    fn from(e: EnvelopeDecodeError) -> Self {
        IndexParseError::UnknownFile(e)
    }
}

impl From<minicbor::decode::Error> for IndexParseError {
    fn from(_: minicbor::decode::Error) -> Self {
        IndexParseError::V4IndexDeserializeError
    }
}
