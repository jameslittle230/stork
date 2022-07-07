#![allow(dead_code)]

pub mod search;
mod tree;

use bytes::Bytes;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::config::{OutputConfig, TitleBoost};

pub(crate) use tree::CharEdgeTree;

// #[derive(Debug, Clone, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
// struct QueryTreeRemoteDestination {
//     partial_index_name: String,
// }

// #[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
// enum QueryTreeChildDestination {
//     Local(Box<QueryTreeNode>),
//     Remote(Box<QueryTreeRemoteDestination>),
// }

/**
 * A query tree node has multiple children, each represented by the character
 * that gets consumed when the map's key is matched against the query. Some
 * values i
 */
// type QueryTreeChildren = BTreeMap<char, QueryTreeChildDestination>;

/**
Serializing this data structure with a specific serializer will vend a
binary blob which, when packaged in a Stork Index envelope, will be a valid
Stork Index file.
 */
#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
pub(crate) struct IndexDiskRepresentation {
    /**
    The root of a radix tree for all words in the document. Each node
    in the tree (even if it has children) points to a set of query results
    that get displayed when that word is searched for.
    */
    pub(crate) query_tree: CharEdgeTree<QueryResultIndex>,

    /**
    Represents a possible search result.
    */
    pub(crate) query_results: Vec<QueryResult>,

    /**
    The documents that have been indexed.
    */
    pub(crate) documents: Vec<Document>,
    pub(crate) settings: Settings,
    // metadata_keys: Vec<String>
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

#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
pub(crate) struct PartialIndexDiskRepresentation {
    pub(crate) partial_index_name: String,
    pub(crate) query_tree: CharEdgeTree<QueryResultIndex>,
    pub(crate) query_results: Vec<QueryResult>,
}

/**
A `QueryResult` is something that can be searched for. Matches will come from
the contents of a document, the title of a document, or the value of metadata
for one or more documents (ex: searching for "Piper" should return all documents
who have a metadata value where the key is "Author" and the value is "Jessica Piper")
 */
#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub(crate) enum QueryResult {
    DocumentContentsExcerpt(Excerpt),
    TitleExcerpt(TitleExcerpt),
    MetadataValue(MetadataValue),
}

/**
A document that is indexed, serialized in a V4 index.

Excerpts belong to exactly one document, and are grouped by document in the
visible search results.
 */
#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
pub(crate) struct Document {
    pub(crate) title: String,
    pub(crate) contents: String,
    pub(crate) url: String,
    pub(crate) metadata: Vec<MetadataEntry>,
}

/**
 * User-defined key/value pair that is indexed and searchable.
 */
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
pub(crate) struct Excerpt {
    pub(crate) document_id: DocumentIndex,
    pub(crate) contents_character_offset: CharacterOffset,
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

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn build_empty_index_disk_representation() {
        let index = IndexDiskRepresentation::default();
        let bytes = index.to_bytes();
        dbg!(&bytes.len());
        dbg!(&bytes);
    }

    #[test]
    fn build_small_index_disk_representation() {
        let mut query_tree: CharEdgeTree<QueryResultIndex> = CharEdgeTree::default();
        query_tree.push_value_for_word("according", 0);

        let index = IndexDiskRepresentation {
        query_tree,
        query_results: vec![QueryResult::DocumentContentsExcerpt(Excerpt {
            document_id: 0,
            contents_character_offset: 0,
        })],
        documents: vec![Document {
            title: "This is a document title.".to_string(),
            contents: "According to all known laws of aviation, there is no way that a bee should be able to fly. Its wings are too small to get its fat little body off the ground. The bee, of course, flies anyway because bees don't care what humans think is impossible.".to_string(),
            url: "https://www.example.com/".to_string(),
            metadata: vec![
                MetadataEntry {
                    key: "author".to_string(),
                    value: "John Doe".to_string(),
                },
            ],
        }],
        settings: Settings::default(),
    };
        let bytes = index.to_bytes();
        dbg!(&bytes.len());
    }
}
