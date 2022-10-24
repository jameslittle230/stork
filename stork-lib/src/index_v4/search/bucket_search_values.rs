use std::collections::HashMap;

use crate::{
    index_v4::{
        tree::CharactersRemaining, DocumentContentsExcerpt, DocumentIndex, QueryResult,
        TitleExcerpt,
    },
    search_value::V4SearchValue,
};

#[derive(Debug, Clone)]
pub(super) struct DocumentContentsExcerptWithCharsRemaining {
    pub(super) excerpt: DocumentContentsExcerpt,
    pub(super) characters_remaining: CharactersRemaining,
}

#[derive(Debug, Clone)]
pub(super) struct TitleExcerptWithCharsRemaining {
    pub(super) excerpt: TitleExcerpt,
    pub(super) characters_remaining: CharactersRemaining,
}

/// Excerpts sorted by their type, either `DocumentContentsExcerpt` or `TitleExcerpt`.
/// Each excerpt is accompanied by their CharactersRemaining
#[derive(Debug, Clone)]
pub(super) struct SortedExcerpts {
    pub(super) document_contents_excerpts: Vec<DocumentContentsExcerptWithCharsRemaining>,
    pub(super) title_excerpts: Vec<TitleExcerptWithCharsRemaining>,
}

pub(super) fn bucket_search_values_by_document(
    values: Vec<&V4SearchValue>,
) -> HashMap<DocumentIndex, SortedExcerpts> {
    values
        .iter()
        .fold(HashMap::new(), |mut accumulator, value| {
            let V4SearchValue {
                term,
                result,
                chars_remaining,
            } = value;

            match result {
                QueryResult::DocumentContentsExcerpt(excerpt) => {
                    accumulator
                        .entry(excerpt.document_id)
                        .and_modify(|entry| {
                            entry.document_contents_excerpts.push(
                                DocumentContentsExcerptWithCharsRemaining {
                                    excerpt: excerpt.clone(),
                                    characters_remaining: *chars_remaining,
                                },
                            )
                        })
                        .or_insert(SortedExcerpts {
                            document_contents_excerpts: vec![
                                DocumentContentsExcerptWithCharsRemaining {
                                    excerpt: excerpt.clone(),
                                    characters_remaining: *chars_remaining,
                                },
                            ],
                            title_excerpts: vec![],
                        });
                }

                QueryResult::TitleExcerpt(excerpt) => {
                    accumulator
                        .entry(excerpt.document_id)
                        .and_modify(|entry| {
                            entry.title_excerpts.push(TitleExcerptWithCharsRemaining {
                                excerpt: excerpt.clone(),
                                characters_remaining: *chars_remaining,
                            })
                        })
                        .or_insert(SortedExcerpts {
                            document_contents_excerpts: vec![],
                            title_excerpts: vec![TitleExcerptWithCharsRemaining {
                                excerpt: excerpt.clone(),
                                characters_remaining: *chars_remaining,
                            }],
                        });
                }

                QueryResult::MetadataValue(_) => {
                    panic!("Need to handle metadata value in search!") // TODO
                }
            }

            accumulator
        })
}
