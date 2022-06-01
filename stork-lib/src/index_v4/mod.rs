#![allow(dead_code)]

use std::collections::BTreeMap;

use bytes::Bytes;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::config::{OutputConfig, TitleBoost};

#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
struct QueryTreeRemoteDestination {
    partial_index_name: String,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
enum QueryTreeChildDestination {
    Local(QueryTreeNode),
    Remote(QueryTreeRemoteDestination),
}

/**
 * A query tree node has multiple children, each represented by the character
 * that gets consumed when the map's key is matched against the query. Some
 * values i
 */
type QueryTreeChildren = BTreeMap<char, QueryTreeChildDestination>;

/**
 * Serializing this data structure with a specific serializer will vend a
 * binary blob which, when packaged in a Stork Index envelope, will be a valid
 * Stork Index file.
 */
#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
struct IndexDiskRepresentation {
    /**
     * The root of a radix tree for all words in the document. Each node
     * in the tree (even if it has children) points to a set of query results
     * that get displayed when that word is searched for.
     */
    query_tree: QueryTreeChildren,

    /**
     * Represents a possible search result.
     */
    query_results: Vec<QueryResult>,

    /**
     * The documents that have been indexed. Each query
     */
    documents: Vec<Document>,
    settings: Settings,
    // metadata_keys: Vec<String>
}

impl IndexDiskRepresentation {
    fn to_bytes(&self) -> Bytes {
        Bytes::from(rmp_serde::to_vec(self).unwrap())
    }
}

#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
struct PartialIndexDiskRepresentation {
    partial_index_name: String,
    query_tree: QueryTreeNode,
    query_results: Vec<QueryResult>,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
enum QueryResult {
    DocumentSourceExcerpt(Excerpt),
    TitleExcerpt(TitleExcerpt),
    MetadataValue(MetadataValue),
}

/**
 * A document that is indexed. Excerpts belong to exactly one document, and
 * are grouped by document in the visible search results.
 */
#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
struct Document {
    title: String,
    contents: String,
    url: String,
    metadata: Vec<MetadataEntry>,
}

/**
 * User-defined key/value pair that is indexed and searchable.
 */
#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
struct MetadataEntry {
    key: String,
    value: String,
}

/**
 * An extracted piece of a document's contents
 */
#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
struct Excerpt {
    document_id: u64,
    contents_character_offset: u64,
}

#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
struct TitleExcerpt {
    document_id: u64,
    title_character_offset: u64,
}

#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
struct MetadataValue {
    document_id: u64,
    metadata_index: u64,
}

#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
struct QueryTreeNode {
    best_query_results: Vec<u64>,
    total_children_count: u64,
    children: QueryTreeChildren,
}

#[derive(Debug, Clone, SmartDefault, PartialEq, PartialOrd, Serialize, Deserialize)]
struct Settings {
    url_prefix: String,

    title_boost: TitleBoost,

    #[default(OutputConfig::default().excerpt_buffer)]
    excerpt_buffer: u8,

    #[default(OutputConfig::default().excerpts_per_result)]
    excerpts_per_result: u8,

    #[default(OutputConfig::default().displayed_results_count)]
    displayed_results_count: u8,
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
        let index = IndexDiskRepresentation {
            query_tree: BTreeMap::from([(
                'a',
                QueryTreeChildDestination::Local(QueryTreeNode {
                    best_query_results: vec![1, 2, 3],
                    total_children_count: 25,
                    children: BTreeMap::from([
                        ('c', QueryTreeChildDestination::Local(QueryTreeNode {
                            best_query_results: vec![1, 2, 3],
                            total_children_count: 25,
                            children: BTreeMap::new()
                        }))
                    ]),
                }),
            )]),
            query_results: vec![QueryResult::DocumentSourceExcerpt(Excerpt {
                document_id: 0,
                contents_character_offset: 1234,
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
