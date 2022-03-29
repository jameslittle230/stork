use crate::{IndexMetadata, StorkIndex};

#[cfg(feature = "search-v2")]
use crate::V2Index;

#[cfg(feature = "search-v3")]
use crate::V3Index;

#[cfg(feature = "search-v3")] // TODO: Change to v4
use crate::V4Index;

#[derive(Debug)]
pub enum DeserializedIndex {
    #[cfg(feature = "search-v2")]
    V2(V2Index),

    #[cfg(feature = "search-v3")]
    V3(V3Index),

    #[cfg(feature = "search-v3")]
    V4(V4Index),
}

impl DeserializedIndex {
    pub fn metadata(&self) -> Option<IndexMetadata> {
        match self {
            #[cfg(feature = "search-v2")]
            DeserializedIndex::V2(index) => Some(index.metadata()),

            #[cfg(feature = "search-v3")]
            DeserializedIndex::V3(index) => Some(index.metadata()),

            #[cfg(feature = "search-v3")] // TODO: Change to v4
            DeserializedIndex::V4(index) => Some(index.metadata()),

            _ => None,
        }
    }
}
