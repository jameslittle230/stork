use super::bucket_search_values::DocumentContentsExcerptWithCharsRemaining;

#[derive(Debug, Clone)]
pub(super) struct GroupedDocumentExcerpts {
    pub(super) starting_character_index: usize,
    pub(super) score: usize,
    pub(super) excerpts: Vec<DocumentContentsExcerptWithCharsRemaining>,
}

pub(super) fn group_document_excerpts(
    document_excerpts: &Vec<DocumentContentsExcerptWithCharsRemaining>,
    max_group_length: usize,
) -> Vec<GroupedDocumentExcerpts> {
    let mut grouped_document_excerpts: Vec<GroupedDocumentExcerpts> =
        document_excerpts.iter().fold(
            vec![],
            |mut acc,
             DocumentContentsExcerptWithCharsRemaining {
                 excerpt,
                 characters_remaining,
             }| {
                match acc.last_mut() {
                    Some(last_group) => {
                        if excerpt.contents_character_offset
                            < last_group.starting_character_index + 100
                        {
                            last_group
                                .excerpts
                                .push(DocumentContentsExcerptWithCharsRemaining {
                                    excerpt: excerpt.clone(),
                                    characters_remaining: *characters_remaining,
                                });
                            last_group.score +=
                                (10_u8.saturating_sub(*characters_remaining)) as usize;
                        } else {
                            acc.push(GroupedDocumentExcerpts {
                                starting_character_index: excerpt.contents_character_offset,
                                score: 10,
                                excerpts: vec![DocumentContentsExcerptWithCharsRemaining {
                                    excerpt: excerpt.clone(),
                                    characters_remaining: *characters_remaining,
                                }],
                            })
                        }
                    }
                    None => acc.push(GroupedDocumentExcerpts {
                        starting_character_index: excerpt.contents_character_offset,
                        score: 10,
                        excerpts: vec![DocumentContentsExcerptWithCharsRemaining {
                            excerpt: excerpt.clone(),
                            characters_remaining: *characters_remaining,
                        }],
                    }),
                }

                acc
            },
        );

    grouped_document_excerpts.sort_by_key(|gde| usize::MAX - gde.score);

    grouped_document_excerpts
}
