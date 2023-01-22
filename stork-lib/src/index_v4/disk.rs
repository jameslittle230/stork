use std::collections::HashMap;

use bytes::Bytes;
use minicbor::Decode;

#[cfg(feature = "build")]
use minicbor::Encode;

use super::{
    tree::{Arena, ArenaId, Tree},
    ChunkId, ContentSlicePointer, DocumentId, Fields, QueryResult, Settings,
};

#[derive(Decode)]
#[cfg_attr(feature = "build", derive(Encode, Debug))]
pub(crate) struct DocumentMetadata {
    #[n(0)]
    pub(crate) id: DocumentId,

    #[n(1)]
    pub(crate) title: String,

    #[n(2)]
    pub(crate) url: String,

    #[n(3)]
    pub(crate) content_length: usize,

    #[n(4)]
    pub(crate) metadata: Fields,
}

#[derive(Decode)]
#[cfg_attr(feature = "build", derive(Encode, Debug))]
pub(crate) struct RootChunk {
    #[n(0)]
    pub(crate) settings: Settings,

    #[n(6)]
    pub(crate) arena_map: HashMap<ArenaId, ChunkId>,

    #[n(7)]
    pub(crate) content_slice_map: HashMap<ContentSlicePointer, ChunkId>,

    #[n(1)]
    pub(crate) document_metadata: HashMap<usize, DocumentMetadata>,

    #[n(2)]
    pub(crate) query_result_tree_root: Tree<QueryResult>, // TODO: duplicative with the arenas in initial_data; remove this and replace with a pointer to the root arena

    // #[n(5)]
    // pub(crate) query_results: HashMap<usize, QueryResult>,
    #[n(3)]
    pub(crate) chunk_ids: Vec<ChunkId>,

    #[n(4)]
    pub(crate) initial_data: Option<Chunk>,
}

impl TryFrom<Bytes> for RootChunk {
    type Error = minicbor::decode::Error;

    fn try_from(bytes: Bytes) -> Result<Self, Self::Error> {
        minicbor::decode(&bytes)
    }
}

#[cfg(feature = "build")]
impl RootChunk {
    pub(crate) fn to_bytes(&self) -> Bytes {
        let mut buf = Vec::new();
        minicbor::encode(self, &mut buf).unwrap();
        Bytes::from(buf)
    }
}

#[derive(Decode, Clone)]
#[cfg_attr(feature = "build", derive(Encode, Debug))]
pub(crate) struct ContentSlice {
    #[n(0)]
    pub(crate) document_id: DocumentId,
    #[n(1)]
    pub(crate) start_char_offset: usize,
    #[n(2)]
    pub(crate) contents: String,
}

#[derive(Decode, Clone)]
#[cfg_attr(feature = "build", derive(Encode, Debug))]
pub(crate) struct Chunk {
    #[n(0)]
    pub(crate) id: ChunkId,
    #[n(1)]
    pub(crate) arenas: Vec<Arena<QueryResult>>,

    #[n(2)]
    pub(crate) content_slices: Vec<ContentSlice>,
}

impl TryFrom<Bytes> for Chunk {
    type Error = minicbor::decode::Error;

    fn try_from(bytes: Bytes) -> Result<Self, Self::Error> {
        minicbor::decode(&bytes)
    }
}

#[cfg(feature = "build")]
impl Chunk {
    pub(crate) fn to_bytes(&self) -> Bytes {
        let mut buf = Vec::new();
        minicbor::encode(self, &mut buf).unwrap();
        Bytes::from(buf)
    }
}
