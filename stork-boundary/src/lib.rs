mod input;
mod output;

pub use input::{IndexVersioningError, VersionedIndex};
pub use output::{
    Entry, Excerpt, HighlightRange, IndexMetadata, InternalWordAnnotation, Output, Result,
};
