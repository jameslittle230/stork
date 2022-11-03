use std::{collections::HashSet, ops::Div};

use itertools::Itertools;

use crate::search_query::SearchTerm;

use super::bucket_search_values::DocumentContentsExcerptWithCharsRemaining;

#[derive(Debug, Clone)]
pub(super) struct GroupedDocumentExcerpts {
    pub(super) starting_character_index: usize,
    pub(super) search_terms: HashSet<SearchTerm>,
    pub(super) excerpts: Vec<DocumentContentsExcerptWithCharsRemaining>,
}

impl GroupedDocumentExcerpts {
    fn new(excerpts: Vec<DocumentContentsExcerptWithCharsRemaining>) -> Self {
        let starting_character_index = &excerpts.first().unwrap().excerpt.contents_character_offset;
        let mut search_terms = HashSet::new();
        for excerpt in &excerpts {
            _ = search_terms.insert(excerpt.search_term.clone());
        }

        Self {
            starting_character_index: *starting_character_index,
            search_terms,
            excerpts,
        }
    }

    pub(super) fn score(&self) -> usize {
        // let mut sorted_excerpts = self.excerpts.clone();
        // sorted_excerpts.sort_by_key(|e| e.excerpt.contents_character_offset);

        let excerpts_count = self.excerpts.iter().fold(0, |acc, excerpt| {
            acc + 20_u8.saturating_sub(excerpt.characters_remaining) as usize
        });

        #[rustfmt::skip]
        let first_offset = self.excerpts.first().unwrap().excerpt.contents_character_offset;

        #[rustfmt::skip]
        let last_offset = self.excerpts.last().unwrap().excerpt.contents_character_offset;

        match self.excerpts.len().cmp(&1) {
            std::cmp::Ordering::Less => panic!(),
            std::cmp::Ordering::Equal => {
                debug_assert!(last_offset == first_offset);
                excerpts_count
            }
            std::cmp::Ordering::Greater => {
                debug_assert!(last_offset > first_offset);

                let char_range = last_offset - first_offset;

                ((excerpts_count * 2_usize.pow(self.search_terms.len() as u32)) as f32
                    * (10_f32.div(char_range as f32))) as usize
            }
        }
    }
}

pub(super) fn group_document_excerpts(
    document_excerpts: Vec<DocumentContentsExcerptWithCharsRemaining>,
    max_group_length: usize,
) -> Vec<GroupedDocumentExcerpts> {
    // Each item in this vec corresponds to a single output excerpt, which might contain multiple words
    let mut groups: Vec<(usize, usize)> = vec![];

    let mut sorted_document_excerpts = document_excerpts;
    sorted_document_excerpts.sort_by_key(|e| e.excerpt.contents_character_offset);

    let len = sorted_document_excerpts.len();

    let mut i = 0;
    while i < len {
        let mut j = i;
        while j < len - 1
            && sorted_document_excerpts[j + 1]
                .excerpt
                .contents_character_offset
                < sorted_document_excerpts[i]
                    .excerpt
                    .contents_character_offset
                    + max_group_length
        {
            j += 1;
        }

        groups.push((i, j));
        i = j + 1;
    }

    let mut grouped_excerpts = groups
        .iter()
        .map(|(start_index, end_index)| {
            // let mut excerpts = Vec::with_capacity(end_index - start_index + 1);
            let slice = &sorted_document_excerpts[*start_index..=*end_index];
            let excerpts = Vec::from(slice);
            GroupedDocumentExcerpts::new(excerpts)
        })
        .collect_vec();

    grouped_excerpts.sort_by_cached_key(|gde| usize::MAX - (gde.score()));
    grouped_excerpts.truncate(50);
    grouped_excerpts
}
