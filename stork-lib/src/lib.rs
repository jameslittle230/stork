use bytes::Bytes;
use lazy_static::lazy_static;

use std::collections::HashMap;
use std::convert::TryFrom;
use std::sync::Mutex;
use thiserror::Error;

#[cfg(feature = "build-v3")]
use {
    num_format::{Locale, ToFormattedString},
    std::fmt::Display,
};

pub type Fields = HashMap<String, String>;

pub trait StorkIndex: TryFrom<Bytes> {
    fn metadata(&self) -> IndexMetadata;
}

mod output;
pub use output::{
    Entry, Excerpt, HighlightRange, IndexMetadata, InternalWordAnnotation, Output, Result,
};

mod input;
use input::{DeserializedIndex, IndexVersioningError, VersionedBlob};

mod stopwords;
use stopwords::STOPWORDS as stopwords;

mod config;
pub use config::{Config, ConfigReadError};

#[cfg(feature = "search-v2")]
mod index_v2;

#[cfg(feature = "search-v3")]
mod index_v3;

#[cfg(feature = "search-v3")] // TODO: Change to v4
mod index_v4;

#[cfg(feature = "search-v2")]
use {index_v2::search as V2Search, index_v2::Index as V2Index};

#[cfg(feature = "search-v3")]
use {index_v3::search as V3Search, index_v3::Index as V3Index};

#[cfg(feature = "search-v3")] // TODO: Change to v4
use index_v4::V4Index;

#[cfg(feature = "build-v3")]
pub use index_v3::DocumentError;

#[cfg(feature = "build-v3")]
use {
    index_v3::build as V3Build, index_v3::BuildResult as V3BuildResult,
    index_v3::IndexGenerationError,
};

#[cfg(feature = "build-v3")] // TODO: Change to v4
use {
    index_v4::build as V4Build, index_v4::BuildError as V4BuildError,
    index_v4::BuildWarning as V4BuildWarning,
};

// We can't pass a parsed index over the WASM boundary so we store the parsed indices here
lazy_static! {
    static ref INDEX_CACHE: Mutex<HashMap<String, DeserializedIndex>> = Mutex::new(HashMap::new());
}

/**
 * An error that may occur when trying to parse an index file.
 */
#[derive(Error, Debug)]
pub enum IndexParseError {
    #[error("{0}")]
    VersioningError(#[from] IndexVersioningError),

    #[error("Could not parse index, despite knowing the version.")]
    ParseError(),

    #[error("{0}")]
    V3SerdeError(String),
}

#[allow(unreachable_patterns)]
pub fn index_from_bytes(bytes: Bytes) -> core::result::Result<DeserializedIndex, IndexParseError> {
    let versioned = VersionedBlob::try_from(bytes)?;

    match versioned {
        #[cfg(feature = "search-v2")]
        VersionedBlob::V2(bytes) => V2Index::try_from(bytes).map(DeserializedIndex::V2),

        #[cfg(feature = "search-v3")]
        VersionedBlob::V3(bytes) => V3Index::try_from(bytes).map(DeserializedIndex::V3),

        #[cfg(feature = "search-v3")] // TODO: Change to v4
        VersionedBlob::V4(bytes) => V4Index::try_from(bytes).map(DeserializedIndex::V4),

        _ => Err(IndexParseError::ParseError()),
    }
}
#[derive(Debug, Error)]
pub enum BuildError {
    #[error("{0}")]
    ConfigReadError(#[from] ConfigReadError),

    #[error("The Stork binary was not compiled with the ability to build indexes. Please recompile with the `build_v3` feature enabled.")]
    BinaryNotBuiltWithFeature,

    #[error("{0}")]
    #[cfg(feature = "build-v3")]
    IndexGenerationError(#[from] IndexGenerationError),

    #[error("{0}")]
    V4BuildError(#[from] V4BuildError),
}

#[cfg(feature = "build-v3")]
#[derive(Debug)]
pub struct IndexDescription {
    pub entries_count: usize,
    pub tokens_count: usize,
    pub index_size_bytes: usize,
    pub warnings: Vec<DocumentError>,
}

#[cfg(feature = "build-v3")]
impl From<&V3BuildResult> for IndexDescription {
    fn from(build_result: &V3BuildResult) -> Self {
        Self {
            entries_count: build_result.index.entries_len(),
            tokens_count: build_result.index.tokens_count(),
            index_size_bytes: Bytes::from(&build_result.index).len(),
            warnings: build_result.errors.clone(),
        }
    }
}

#[cfg(feature = "build-v3")]
impl Display for IndexDescription {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            r#"{}Index stats:
  - {} entries
  - {} search terms
  - {} bytes per entry
  - {} bytes per search term"#,
            if self.warnings.is_empty() {
                "".to_string()
            } else {
                DocumentError::display_list(&self.warnings) + "\n"
            },
            self.entries_count.to_formatted_string(&Locale::en),
            self.tokens_count.to_formatted_string(&Locale::en),
            (self.index_size_bytes / self.entries_count).to_formatted_string(&Locale::en),
            (self.index_size_bytes / self.tokens_count).to_formatted_string(&Locale::en),
        ))
    }
}

#[cfg(feature = "build-v3")]
pub struct BuildOutput {
    pub bytes: Bytes,
    pub metadata: IndexMetadata,
}

#[cfg(not(feature = "build-v3"))]
pub fn build_index(_config: &Config) -> core::result::Result<(), BuildError> {
    Err(BuildError::BinaryNotBuiltWithFeature)
}

#[cfg(feature = "build-v3")] // TODO: Change to v4
pub fn build_index(config: &Config) -> core::result::Result<BuildOutput, BuildError> {
    use index_v4::CompressionMethod;

    let build_output = V4Build(config)?;
    let compression_method = CompressionMethod::BZip2;
    let bytes = build_output.index.into_bytes(compression_method);
    let metadata = build_output.metadata;
    Ok(BuildOutput { bytes, metadata })
}

pub fn register_index(
    name: &str,
    bytes: Bytes,
) -> core::result::Result<IndexMetadata, IndexParseError> {
    let mut deserialized_index = index_from_bytes(bytes)?;

    // If we're dealing with a v4 index, let's unfurl it into a v3 index
    // so we don't have to pay that for every search op
    if let DeserializedIndex::V4(v4_index) = deserialized_index {
        let v3_index = V3Index::from(&v4_index);
        deserialized_index = DeserializedIndex::V3(v3_index)
    }

    if let Some(metadata) = deserialized_index.metadata() {
        INDEX_CACHE
            .lock()
            .unwrap()
            .insert(name.to_string(), deserialized_index);

        Ok(metadata)
    } else {
        // TODO: Model this error better
        Err(IndexParseError::ParseError())
    }
}

#[derive(Debug, Error)]
pub enum SearchError {
    #[error("{0}")]
    IndexParseError(#[from] IndexParseError),

    #[error("The index is not supported. You might need to recompile Stork with a different set of features enabled.")]
    IndexVersionNotSupported,

    #[error(
        "Index `{0}` has not been registered. You need to register the index before performing searches with it."
    )]
    IndexNotInCache(String),
}

pub fn search_from_cache(key: &str, query: &str) -> core::result::Result<Output, SearchError> {
    let cache = INDEX_CACHE.lock().unwrap();
    let parsed = match cache.get(key) {
        Some(parsed) => parsed,
        None => return Err(SearchError::IndexNotInCache(key.to_string())),
    };

    match parsed {
        #[cfg(feature = "search-v2")]
        DeserializedIndex::V2(index) => Ok(V2Search(index, query)),

        #[cfg(feature = "search-v3")]
        DeserializedIndex::V3(index) => Ok(V3Search(index, query)),

        #[cfg(feature = "search-v3")]
        DeserializedIndex::V4(index) => {
            if cfg!(debug_assertions) {
                eprintln!("Calling search on a V4 index - this should never happen naturally")
            }

            let v3_index = V3Index::from(index);

            Ok(V3Search(&v3_index, query))
        }

        #[cfg(not(any(feature = "search-v2", feature = "search-v3")))]
        DeserializedIndex::Unknown => Err(SearchError::IndexVersionNotSupported),
    }
}

#[allow(unused_variables)]
pub fn search(index: Bytes, query: &str) -> core::result::Result<Output, SearchError> {
    let index = index_from_bytes(index)?;

    #[allow(unreachable_patterns)]
    match index {
        #[cfg(feature = "search-v3")] // TODO: Change to v4
        DeserializedIndex::V4(index) => {
            let v3_index = V3Index::from(&index);
            Ok(V3Search(&v3_index, query))
        }

        #[cfg(feature = "search-v3")]
        DeserializedIndex::V3(index) => Ok(V3Search(&index, query)),

        #[cfg(feature = "search-v2")]
        DeserializedIndex::V2(index) => Ok(V2Search(&index, query)),

        _ => Err(SearchError::IndexVersionNotSupported),
    }
}
