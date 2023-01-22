use std::collections::HashMap;

pub(crate) mod search;

mod disk;
mod tree;

use disk::ContentSlice;
pub(crate) use disk::{Chunk, RootChunk};

#[cfg(feature = "build")]
use itertools::Itertools;

use crate::{search_query::SearchTerm, Fields};

#[cfg(feature = "build")]
use crate::build::parse_document::DocumentParseValue;

use minicbor::Decode;

#[cfg(feature = "build")]
use minicbor::Encode;

#[cfg(feature = "build")]
use crate::build_config::Config;

#[cfg(feature = "build")]
#[allow(unused_imports)]
use crate::envelope::{Envelope, Prefix};

use self::{
    disk::DocumentMetadata,
    tree::{ArenaId, Tree},
};

#[derive(Clone)]
pub(crate) enum V4SearchValue {
    SearchResult {
        term: SearchTerm,
        result: tree::TreeRetrievalValue<QueryResult>,
    },

    Filter {
        term: SearchTerm,
    },
}

/// In-memory representation of a search index.
#[cfg_attr(feature = "build", derive(Debug))]
pub(crate) struct Index {
    /// Data coming from the build process that needs to be encoded into the index
    pub(crate) settings: Settings,

    /// Correlates tree arenas with the chunk they live in.
    ///
    /// This map aids in the scenario where we try to load a node from the tree,
    /// but that subtree hasn't been loaded yet. Often, that arena will belong
    /// to a different chunk that hasn't been fetched; this map provides context
    /// so that the caller knows which chunk it needs to load from the remote source.
    pub(crate) arena_map: HashMap<ArenaId, ChunkId>,

    /// Correlates all content slice pointers with the chunk they live in.
    ///
    /// This map aids in the scenario where we try to load a node from the tree,
    /// but that subtree hasn't been incorporated yet. Often, that arena will belong
    /// to a different chunk that hasn't been fetched; this map provides context
    /// so that the caller knows which chunk it needs to load from the remote source.
    pub(crate) content_slice_map: HashMap<ContentSlicePointer, ChunkId>,

    /// A list of the remote chunks that have been loaded into this data structure
    pub(crate) loaded_chunk_ids: Vec<ChunkId>,

    /// The documents known by the index.
    ///
    /// When chunks have been loaded, the Document values in this map also contain
    /// all known content slices.
    pub(crate) documents: HashMap<DocumentId, Document>,

    pub(crate) query_result_tree: Tree<QueryResult>,
}

#[derive(Clone, Decode, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "build", derive(Encode, Debug))]
pub struct ContentSlicePointer {
    #[n(0)]
    document_id: DocumentId,
    #[n(1)]
    beginning_char_offset: usize,
    #[n(2)]
    end_char_offset: usize,
}

#[derive(Clone, Decode)]
#[cfg_attr(feature = "build", derive(Encode, Debug))]
pub enum TitleBoost {
    #[n(0)]
    Minimal,
    #[n(1)]
    Moderate,
    #[n(2)]
    Large,
    #[n(3)]
    Ridiculous,
}

#[cfg(feature = "build")]
impl From<crate::build_config::TitleBoost> for TitleBoost {
    fn from(other: crate::build_config::TitleBoost) -> Self {
        match other {
            crate::build_config::TitleBoost::Minimal => TitleBoost::Minimal,
            crate::build_config::TitleBoost::Moderate => TitleBoost::Moderate,
            crate::build_config::TitleBoost::Large => TitleBoost::Large,
            crate::build_config::TitleBoost::Ridiculous => TitleBoost::Ridiculous,
        }
    }
}

#[derive(Clone, Decode)]
#[cfg_attr(feature = "build", derive(Encode, Debug))]
pub(crate) struct Settings {
    #[n(0)]
    pub(crate) url_prefix: String,
    #[n(1)]
    pub(crate) title_boost: TitleBoost,
    #[n(2)]
    pub(crate) minimum_query_length: u8,
    #[n(3)]
    pub(crate) excerpt_buffer: u8,
    #[n(4)]
    pub(crate) excerpts_per_result: u8,
    #[n(5)]
    pub(crate) displayed_results_count: u8,
}

#[cfg(feature = "build")]
impl Settings {
    pub(crate) fn new_from_config(config: &Config) -> Self {
        Self {
            url_prefix: config.input.url_prefix.clone(),
            title_boost: config.input.title_boost.clone().into(),
            minimum_query_length: config.output.minimum_query_length,
            excerpt_buffer: config.output.excerpt_buffer,
            excerpts_per_result: config.output.excerpts_per_result,
            displayed_results_count: config.output.displayed_results_count,
        }
    }
}

type DocumentId = usize;
type ChunkId = usize;

#[derive(Clone, Decode)]
#[cfg_attr(feature = "build", derive(Encode, Debug))]
pub(crate) enum QueryResult {
    #[n(0)]
    ContentsExcerpt(#[n(0)] ContentsExcerpt),
    #[n(1)]
    TitleExcerpt(#[n(0)] TitleExcerpt),
}

#[derive(Clone, Decode)]
#[cfg_attr(feature = "build", derive(Encode, Debug))]
pub(crate) struct ContentsExcerpt {
    #[n(0)]
    pub(crate) document_id: DocumentId,
    #[n(1)]
    pub(crate) byte_offset: usize,
    #[n(2)]
    pub(crate) importance: f64,
    #[n(3)]
    pub(crate) url_suffix: Option<String>,
}
#[derive(Clone, Decode)]
#[cfg_attr(feature = "build", derive(Encode, Debug))]
pub(crate) struct TitleExcerpt {
    #[n(0)]
    pub(crate) document_id: DocumentId,
    #[n(1)]
    pub(crate) byte_offset: usize,
}

#[cfg_attr(feature = "build", derive(Debug))]
pub struct Document {
    id: DocumentId,
    title: String,
    url: String,
    metadata: Fields,
    contents: Vec<ContentSlice>, // TODO: Find a better way of storing content chunks. Ideally this would be some data structure that describes a discontinuous string?
}

impl From<&DocumentMetadata> for Document {
    // TODO: replace with a builder pattern
    fn from(doc_metadata: &DocumentMetadata) -> Self {
        Self {
            id: doc_metadata.id,
            title: doc_metadata.title.clone(),
            url: doc_metadata.url.clone(),
            metadata: doc_metadata.metadata.clone(),
            contents: vec![],
        }
    }
}

#[cfg(feature = "build")]
impl Document {
    pub(crate) fn from_parse_value(id: usize, value: &DocumentParseValue) -> Self {
        Self {
            id,
            title: value.title.clone(),
            url: value.url.clone(),
            metadata: value.fields.clone().unwrap_or_default(),
            contents: vec![ContentSlice {
                document_id: id,
                start_char_offset: 0,
                contents: value.contents.clone(),
            }],
        }
    }
}

impl From<RootChunk> for Index {
    fn from(root: RootChunk) -> Self {
        let mut documents =
            root.document_metadata
                .iter()
                .fold(HashMap::new(), |mut acc, (k, v)| {
                    acc.insert(*k, Document::from(v));
                    acc
                });

        if let Some(data) = &root.initial_data {
            let Chunk {
                id,
                arenas,
                content_slices,
            } = data;

            for slice in content_slices {
                documents
                    .entry(slice.document_id)
                    .and_modify(|entry| entry.contents.push(slice.clone()));
            }
        }

        Self {
            settings: root.settings.clone(),
            arena_map: root.arena_map.clone(),
            content_slice_map: root.content_slice_map.clone(),
            loaded_chunk_ids: vec![],
            documents,
            query_result_tree: root.query_result_tree_root,
        }
    }
}

impl Index {
    pub(crate) fn add_chunk(&mut self, chunk: &Chunk) {
        let Chunk {
            id,
            mut arenas,
            content_slices,
        } = chunk.clone();

        for slice in content_slices {
            self.documents
                .entry(slice.document_id)
                .and_modify(|entry| entry.contents.push(slice.clone()));
        }

        self.query_result_tree.load_arenas(&mut arenas);

        self.loaded_chunk_ids.push(id);
    }
}

#[cfg(feature = "build")]
impl Index {
    pub(crate) fn default_from_config(config: &Config) -> Self {
        Self {
            loaded_chunk_ids: vec![],
            query_result_tree: Tree::new(),
            documents: HashMap::new(),
            settings: Settings::new_from_config(config),
            arena_map: HashMap::new(),
            content_slice_map: HashMap::new(),
        }
    }

    pub(crate) fn to_disk_representation(&self) -> (RootChunk, Vec<Chunk>) {
        let mut document_metadata = HashMap::new();

        self.documents.iter().for_each(|(id, document)| {
            document_metadata.insert(
                *id,
                DocumentMetadata {
                    id: *id,
                    title: document.title.clone(),
                    url: document.url.clone(),
                    content_length: 0,
                    metadata: document.metadata.clone(),
                },
            );
        });

        (
            RootChunk {
                settings: self.settings.clone(),
                arena_map: self.arena_map.clone(),
                content_slice_map: self.content_slice_map.clone(),
                document_metadata,
                query_result_tree_root: self.query_result_tree.clone(),
                chunk_ids: self.loaded_chunk_ids.clone(),
                initial_data: Some(Chunk {
                    id: 0,
                    arenas: vec![],
                    content_slices: self
                        .documents
                        .values()
                        .flat_map(|doc| doc.contents.clone())
                        .collect_vec(),
                }),
            },
            vec![],
        )
    }

    fn to_envelopes(&self) -> Vec<Envelope> {
        let root_rep = self.to_disk_representation().0;
        let bytes = root_rep.to_bytes();
        vec![Envelope {
            prefix: Prefix::StorkV4Root,
            bytes,
        }]
    }
}
