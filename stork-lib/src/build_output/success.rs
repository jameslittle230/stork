use bytes::Bytes;

use super::BuildWarning;

/// The data returned from Stork when a search index is successfully built.
/// This data is agnostic to which index type has been built.
pub struct BuildSuccessValue {
    pub primary_data: Bytes,
    pub sidecar_data: Vec<Bytes>,
    pub statistics: BuildStatistics,
    pub warnings: Vec<BuildWarning>,
}

/// Statistics about the index that was successfully built.
pub struct BuildStatistics {
    pub entries_count: usize,
    pub tokens_count: usize,
    pub index_size_bytes: usize,
}
