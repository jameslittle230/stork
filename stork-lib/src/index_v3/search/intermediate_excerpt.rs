use std::cmp::Ordering;

use crate::{
    fields::Fields,
    index_v3::{EntryIndex, Score, WordListSource},
};

#[derive(Clone, Debug, Default)]
pub(super) struct IntermediateExcerpt {
    pub(super) query: String,
    pub(super) entry_index: EntryIndex,
    pub(super) score: Score,
    pub(super) source: WordListSource,
    pub(super) word_index: usize,
    pub(super) url_prefix: Option<String>,
    pub(super) fields: Fields,
}

impl Ord for IntermediateExcerpt {
    fn cmp(&self, other: &Self) -> Ordering {
        self.score.cmp(&other.score)
    }
}

impl PartialOrd for IntermediateExcerpt {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for IntermediateExcerpt {}

impl PartialEq for IntermediateExcerpt {
    fn eq(&self, other: &Self) -> bool {
        self.entry_index == other.entry_index
    }
}
