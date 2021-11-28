use bytes::Bytes;
use std::convert::TryFrom;
use std::fmt::Display;

use num_format::{Locale, ToFormattedString};

use stork_boundary::{IndexMetadata, IndexVersioningError, Output, VersionedIndex};
use stork_config::Config;
use stork_config::ConfigReadError;

#[cfg(feature = "read-v2")]
pub use stork_index_v2::search as V2Search;

#[cfg(feature = "read-v2")]
pub use stork_index_v2::Index as V2Index;

#[cfg(feature = "read-v3")]
pub use stork_index_v3::search as V3Search;

#[cfg(feature = "read-v3")]
pub use stork_index_v3::Index as V3Index;

#[cfg(feature = "build-v3")]
pub use stork_index_v3::build as V3Build;

#[cfg(feature = "build-v3")]
pub use stork_index_v3::BuildResult as V3BuildResult;

#[cfg(feature = "build-v3")]
pub use stork_index_v3::{DocumentError, IndexGenerationError};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum IndexParseError {
    #[error("{0}")]
    VersioningError(#[from] IndexVersioningError),

    #[error("Could not parse index, despite knowing the version.")]
    ParseError(),

    #[error("{0}")]
    V2Error(String),

    #[error("{0}")]
    V3Error(String),
}

#[derive(Debug)]
pub enum ParsedIndex {
    #[cfg(feature = "read-v2")]
    V2(V2Index),

    #[cfg(feature = "read-v3")]
    V3(V3Index),
}

impl ParsedIndex {
    pub fn get_metadata(&self) -> IndexMetadata {
        match self {
            #[cfg(feature = "read-v2")]
            ParsedIndex::V2(_) => IndexMetadata {
                index_version: "stork-2".to_string(),
            },

            #[cfg(feature = "read-v3")]
            ParsedIndex::V3(_) => IndexMetadata {
                index_version: "stork-3".to_string(),
            },
        }
    }
}

#[allow(unreachable_patterns)]
pub fn index_from_bytes(bytes: Bytes) -> Result<ParsedIndex, IndexParseError> {
    let versioned = VersionedIndex::try_from(bytes)?;

    match versioned {
        #[cfg(feature = "read-v2")]
        VersionedIndex::V2(bytes) => V2Index::try_from(bytes)
            .map_err(|e| IndexParseError::V2Error(e.to_string()))
            .map(ParsedIndex::V2),

        #[cfg(feature = "read-v3")]
        VersionedIndex::V3(bytes) => V3Index::try_from(bytes)
            .map_err(|e| IndexParseError::V3Error(e.to_string()))
            .map(ParsedIndex::V3),

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
}

#[derive(Debug)]
pub struct IndexDescription {
    pub entries_count: usize,
    pub tokens_count: usize,
    pub index_size_bytes: usize,
    pub warnings: String,
}

#[cfg(feature = "build-v3")]
impl From<&V3BuildResult> for IndexDescription {
    fn from(build_result: &V3BuildResult) -> Self {
        Self {
            entries_count: build_result.index.entries_len(),
            tokens_count: build_result.index.search_term_count(),
            index_size_bytes: Bytes::from(&build_result.index).len(),
            warnings: DocumentError::display_list(&build_result.errors),
        }
    }
}

#[cfg(feature = "build-v3")]
impl Display for IndexDescription {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            r#"Index stats:
- {} entries
- {} search terms
- {} bytes ({} bytes per entry, {} bytes per search term)

{}"#,
            self.entries_count.to_formatted_string(&Locale::en),
            self.tokens_count.to_formatted_string(&Locale::en),
            self.index_size_bytes.to_formatted_string(&Locale::en),
            (self.index_size_bytes / self.entries_count).to_formatted_string(&Locale::en),
            (self.index_size_bytes / self.tokens_count).to_formatted_string(&Locale::en),
            &self.warnings
        ))
    }
}

pub struct BuildOutput {
    pub bytes: Bytes,
    pub description: IndexDescription,
}

#[cfg(not(feature = "build-v3"))]
pub fn build_index(_config: &str) -> Result<(IndexDescription, Bytes), BuildError> {
    Err(BuildError::BinaryNotBuiltWithFeature)
}

#[cfg(feature = "build-v3")]
pub fn build_index(config: &str) -> Result<BuildOutput, BuildError> {
    let config = Config::try_from(config)?;
    let result = V3Build(&config)?;
    let description = IndexDescription::from(&result);
    let bytes = Bytes::from(&result.index);
    Ok(BuildOutput { bytes, description })
}

#[derive(Debug, Error)]
pub enum SearchError {
    #[error("{0}")]
    IndexParseError(#[from] IndexParseError),

    #[error("The index is not supported. You might need to recompile Stork with a different set of features enabled.")]
    IndexVersionNotSupported,
}

#[allow(unused_variables)]
pub fn search(index: Bytes, query: &str) -> Result<Output, SearchError> {
    let index = index_from_bytes(index)?;

    #[allow(unreachable_patterns)]
    match index {
        #[cfg(feature = "read-v3")]
        ParsedIndex::V3(index) => Ok(V3Search(&index, query)),

        #[cfg(feature = "read-v2")]
        ParsedIndex::V2(index) => Ok(V2Search(&index, query)),

        _ => Err(SearchError::IndexVersionNotSupported),
    }
}

pub fn get_output_filename_from_old_style_config(config: &str) -> Option<String> {
    let config = Config::try_from(config).ok()?;
    config.output.UNUSED_filename
}
