use std::ops::Div;

use itertools::Itertools;

use crate::{
    index_v4::{ContentsExcerpt, Document, TitleExcerpt},
    search_output::{Excerpt, HighlightRange},
};

#[derive(Clone, Debug)]
pub(super) struct ContentsExcerptWithHighlightLength {
    pub(super) contents_excerpt: ContentsExcerpt,
    pub(super) highlight_length: u8,
    pub(super) characters_remaining: u8,
}

pub(super) struct TitleExcerptWithHighlightLength {
    pub(super) title_excerpt: TitleExcerpt,
    pub(super) highlight_length: u8,
}

pub(super) struct ContentsExcerptGrouping {
    contents_excerpts: Vec<ContentsExcerptWithHighlightLength>,
}

impl ContentsExcerptGrouping {
    pub(super) fn new(contents_excerpt: &ContentsExcerptWithHighlightLength) -> Self {
        Self {
            contents_excerpts: vec![contents_excerpt.clone()],
        }
    }

    pub(super) fn push(&mut self, contents_excerpt: &ContentsExcerptWithHighlightLength) {
        if let Some(latest) = self.contents_excerpts.last_mut() {
            if latest.contents_excerpt.byte_offset == contents_excerpt.contents_excerpt.byte_offset
            {
                // take the longer excerpt
                if latest.highlight_length < contents_excerpt.highlight_length {
                    self.contents_excerpts.pop();
                    self.contents_excerpts.push(contents_excerpt.clone());
                }
            } else {
                self.contents_excerpts.push(contents_excerpt.clone());
            }
        }
    }

    pub(super) fn can_swallow(
        &self,
        other: &ContentsExcerptWithHighlightLength,
        intended_excerpt_length: usize,
    ) -> bool {
        let first_excerpt = self
            .contents_excerpts
            .first()
            .expect("Grouping should always have at least one element");

        let last_excerpt = self
            .contents_excerpts
            .last()
            .expect("Grouping should always have at least one element");

        assert!(first_excerpt.contents_excerpt.byte_offset <= other.contents_excerpt.byte_offset);
        assert!(last_excerpt.contents_excerpt.byte_offset <= other.contents_excerpt.byte_offset);

        let diff = other.contents_excerpt.byte_offset - first_excerpt.contents_excerpt.byte_offset;
        diff < intended_excerpt_length - other.highlight_length as usize
    }

    pub(super) fn score(&self) -> f64 {
        let number_of_distinct_words = self
            .contents_excerpts
            .iter()
            .map(|excerpt| excerpt.contents_excerpt.importance.0.to_bits())
            .unique()
            .count();

        let number_of_words = self.contents_excerpts.len();

        let summed_excerpt_scores = self
            .contents_excerpts
            .iter()
            .map(|excerpt| {
                excerpt.contents_excerpt.importance.0 / (excerpt.characters_remaining + 1) as f64
            })
            .sum::<f64>();

        let average_distance = self
            .contents_excerpts
            .iter()
            .tuple_windows()
            .map(|(a, b)| {
                let distance = b.contents_excerpt.byte_offset - a.contents_excerpt.byte_offset;
                distance as f64
            })
            .sum::<f64>()
            .div(self.contents_excerpts.len() as f64)
            + 1.0;

        number_of_distinct_words as f64 * number_of_words as f64 * summed_excerpt_scores
            / average_distance
    }

    pub(super) fn as_excerpt(
        &self,
        document: &Document,
        intended_excerpt_length: usize,
    ) -> Excerpt {
        let first_byte = self
            .contents_excerpts
            .first()
            .expect("Grouping should always have at least one element")
            .contents_excerpt
            .byte_offset;

        let last_byte = self
            .contents_excerpts
            .last()
            .expect("Grouping should always have at least one element")
            .contents_excerpt
            .byte_offset;

        let spread = last_byte - first_byte;

        let first_byte_buffer = intended_excerpt_length.saturating_sub(spread).div(2);
        let first_output_byte = first_byte.saturating_sub(first_byte_buffer);

        let last_highlight_length = self
            .contents_excerpts
            .last()
            .expect("Grouping should always have at least one element")
            .highlight_length;

        let last_output_byte = std::cmp::min(
            first_output_byte + intended_excerpt_length + last_highlight_length as usize,
            document.contents.first().unwrap().contents.len(),
        );

        assert!(last_output_byte <= document.contents.first().unwrap().contents.len());

        let highlight_ranges = self
            .contents_excerpts
            .iter()
            .map(|excerpt| {
                let beginning = excerpt
                    .contents_excerpt
                    .byte_offset
                    .saturating_sub(first_output_byte);

                HighlightRange {
                    beginning,
                    end: beginning + excerpt.highlight_length as usize,
                }
            })
            .collect_vec();

        Excerpt {
            text: document.contents.first().unwrap().contents[first_output_byte..last_output_byte]
                .to_string(),
            highlight_ranges,
            score: self.score(),
            url_suffix: None,
        }
    }

    pub(super) fn as_debug_string(&self, document: &Document) -> String {
        let excerpt = self.as_excerpt(document, 150);
        format!(
            "{}\n\t{} [{}/{}]",
            excerpt.text,
            self.contents_excerpts
                .iter()
                .map(|e| {
                    let contents = &document.contents.first().unwrap().contents;
                    let beginning_byte = e.contents_excerpt.byte_offset;
                    let end_byte =
                        std::cmp::min(contents.len(), beginning_byte + e.highlight_length as usize);

                    contents[beginning_byte..end_byte].to_string()
                })
                .join(", "),
            excerpt
                .highlight_ranges
                .iter()
                .map(|r| format!("{}-{}", r.beginning, r.end))
                .join(", "),
            self.score()
        )
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::{build::ImportanceValue, index_v4::ContentsExcerpt};

    use super::{ContentsExcerptGrouping, ContentsExcerptWithHighlightLength};

    #[test]
    fn it_stores_the_longest_value_for_two_excerpts_at_the_same_byte_offset() {
        let mut grouping = ContentsExcerptGrouping::new(&ContentsExcerptWithHighlightLength {
            contents_excerpt: ContentsExcerpt {
                document_id: 0,
                url_suffix: None,
                byte_offset: 200,
                importance: ImportanceValue(1.0),
            },
            highlight_length: 10,
            characters_remaining: 0,
        });

        grouping.push(&ContentsExcerptWithHighlightLength {
            contents_excerpt: ContentsExcerpt {
                document_id: 0,
                byte_offset: 200,
                importance: ImportanceValue(0.5),
                url_suffix: None,
            },
            highlight_length: 12,
            characters_remaining: 0,
        });

        assert_eq!(grouping.contents_excerpts.len(), 1);
        assert_eq!(grouping.contents_excerpts[0].highlight_length, 12);
    }
}
