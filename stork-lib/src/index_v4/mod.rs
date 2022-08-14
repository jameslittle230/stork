#![allow(dead_code)]

pub mod search;
mod tree;

use bytes::Bytes;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::config::{OutputConfig, TitleBoost};

pub(crate) use search::search;
pub(crate) use tree::Tree;

/// Serializing this data structure with a specific serializer will vend a
/// binary blob which, when packaged in a Stork Index envelope, will be a valid
/// Stork Index file.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub(crate) struct IndexDiskRepresentation {
    /// The root of a radix tree for all words in the document. Each node
    /// in the tree (even if it has children) points to a set of query results
    /// that get displayed when that word is searched for.
    pub(crate) query_tree: Tree<QueryResultIndex>,

    /// Represents a possible search result.
    pub(crate) query_results: Vec<QueryResult>,

    /// The documents that have been indexed.
    pub(crate) documents: Vec<Document>,

    pub(crate) settings: Settings,
    // metadata_keys: Vec<String> // TODO: Fill this out
    // TODO: LSH hashmap for fuzzy searching? http://jil.im/lsh
}

impl IndexDiskRepresentation {
    pub(crate) fn to_bytes(&self) -> Bytes {
        Bytes::from(rmp_serde::to_vec(self).unwrap())
    }
}

impl TryFrom<&Bytes> for IndexDiskRepresentation {
    type Error = rmp_serde::decode::Error;
    fn try_from(bytes: &Bytes) -> Result<Self, Self::Error> {
        rmp_serde::from_read_ref(bytes)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub(crate) struct PartialIndexDiskRepresentation {
    pub(crate) partial_index_name: String,
    pub(crate) query_tree: Tree<QueryResultIndex>,
    pub(crate) query_results: Vec<QueryResult>,
}

/// A `QueryResult` is something that can be searched for. Matches will come from
/// the contents of a document, the title of a document, or the value of metadata
/// for one or more documents (ex: searching for "Piper" should return all documents
/// who have a metadata value where the key is "Author" and the value is "Jessica Piper")
#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub(crate) enum QueryResult {
    DocumentContentsExcerpt(DocumentContentsExcerpt),
    TitleExcerpt(TitleExcerpt),
    MetadataValue(MetadataValue),
}

/// A document that is indexed, serialized in a V4 index.
///
/// Excerpts belong to exactly one document, and are grouped by document in the
/// visible search results.
#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
pub(crate) struct Document {
    pub(crate) title: String,
    pub(crate) contents: String,
    pub(crate) url: String,
    pub(crate) metadata: Vec<MetadataEntry>,
}

/// User-defined key/value pair that is indexed and searchable.
#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
pub(crate) struct MetadataEntry {
    pub(crate) key: String,
    pub(crate) value: String,
}

pub(crate) type DocumentIndex = usize;
pub(crate) type MetadataIndex = usize;
pub(crate) type QueryResultIndex = usize;
pub(crate) type CharacterOffset = usize;

/// A reference to an extracted piece of a document's contents, serialized in a V4 index.
///
/// This struct contains:
/// * An index into the list of documents
/// * A character offset into the documents' contents
#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
pub(crate) struct DocumentContentsExcerpt {
    pub(crate) document_id: DocumentIndex,
    pub(crate) contents_character_offset: CharacterOffset,
    pub(crate) url_suffix: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
pub(crate) struct TitleExcerpt {
    pub(crate) document_id: DocumentIndex,
    pub(crate) title_character_offset: CharacterOffset,
}

#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
pub(crate) struct MetadataValue {
    pub(crate) document_id: DocumentIndex,
    pub(crate) metadata_entry_index: MetadataIndex, // the index into that document's metadata vec, not some global metadata vec (this doesn't exist)
}

#[derive(Debug, Clone, SmartDefault, PartialEq, PartialOrd, Serialize, Deserialize)]
pub(crate) struct Settings {
    pub(crate) url_prefix: String,

    pub(crate) title_boost: TitleBoost,

    #[default(OutputConfig::default().excerpt_buffer)]
    pub(crate) excerpt_buffer: u8,

    #[default(OutputConfig::default().excerpts_per_result)]
    pub(crate) excerpts_per_result: u8,

    #[default(OutputConfig::default().displayed_results_count)]
    pub(crate) displayed_results_count: u8,
}
