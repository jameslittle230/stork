use itertools::Itertools;

use super::SearchLineItem;

fn merge_two_excerpts(e1: &SearchLineItem, e2: &SearchLineItem) -> Option<SearchLineItem> {
    // dbg!(e1, e2);
    // The character offset (within the document) of the excerpt's first _highlighted_ word.
    let e1_offset = e1.content_offset;

    // The character offset (within the document) of the excerpt's first _highlighted_ word.
    let e2_offset = e2.content_offset;

    let two_thirds_e1_length = (e1.text.len() as f32 * 0.66) as usize;

    // The character offset (within the document) of the excerpt's first _visible_ word
    let e1_beginning = e1_offset.saturating_sub(e1.highlight_ranges.first().beginning);

    // The character offset (within the document) of the excerpt's first _visible_ word
    let e2_beginning = e2_offset.saturating_sub(e2.highlight_ranges.first().beginning);

    assert!(e2_beginning >= e1_beginning);

    if e2_beginning >= e1_beginning.saturating_add(two_thirds_e1_length) {
        return None;
    }

    let beginnings_offset = e2_beginning - e1_beginning;

    // base times height, is one half times the second derivative of the area function, evaluated at
    // len = 93
    //                                          ^658 - 41/47
    // ^617
    //
    // times height, is one half times the second derivative of the area function, evaluated at a
    // len = 90
    //                                            ^665 - 50/60
    // ^615
    // Goal: Get 88

    let merged_text = {
        let e1_len = e1.text.len();
        let mut out = e1.text.clone();
        let second_string = &(e2.text)[(e1_len - (e2_beginning - e1_beginning))..];
        out.push_str(second_string);
        out
    };

    let merged_highlight_ranges = {
        let mut out = e1.highlight_ranges.clone();
        let mut offset_highlight_ranges = e2
            .highlight_ranges
            .clone()
            .iter_mut()
            .map(|range| {
                range.beginning += beginnings_offset;
                range.end += beginnings_offset;
                range.clone()
            })
            .collect();
        out.append(&mut offset_highlight_ranges);
        out
    };

    let highlight_range_score_demotion = merged_highlight_ranges
        .iter()
        .tuple_windows()
        .fold(0, |acc, (range_1, range_2)| {
            acc + (range_2.beginning - range_1.end)
        });

    // If the highlight_range_score_demotion demotes score below 0, we
    // should consider this a failed merge.
    // Note: This behavior is dictated by the starting excerpt scores -
    // raising this score from 50 will make merging more permissive (which might)
    // be what you want!
    (e1.score + e2.score)
        .checked_sub(highlight_range_score_demotion)
        .map(|score| SearchLineItem {
            text: merged_text,
            highlight_ranges: merged_highlight_ranges,
            content_offset: e1.content_offset,
            score,
            fields: e1.fields.clone(), // TODO: Merge fields
            internal_annotations: e1.internal_annotations.clone(), // TODO: Merge internal annotations
            url_suffix: e1.url_suffix.clone(),
        })
}

pub(crate) fn merge_all_excerpts(extended_excerpts: &mut [SearchLineItem]) -> Vec<SearchLineItem> {
    extended_excerpts.sort_by_key(|excerpt| excerpt.content_offset);

    let mut excerpts: Vec<SearchLineItem> = Vec::new();
    excerpts = extended_excerpts
        .iter()
        .fold(excerpts, |mut accumulator, element| {
            let last = accumulator.last_mut();
            match last {
                Some(latest_excerpt) => match merge_two_excerpts(latest_excerpt, element) {
                    Some(merged) => {
                        *latest_excerpt = merged;
                    }
                    None => accumulator.push(element.to_owned()),
                },
                None => {
                    accumulator.push(element.to_owned());
                }
            }

            accumulator
        });

    // TODO: Trim very long excerpt strings, ensuring highlight ranges remain in the center

    excerpts
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use pretty_assertions::assert_eq;

    use vec1::vec1;

    use crate::search_output::HighlightRange;

    use super::*;

    #[test]
    fn it_merges_two_similar_excerpts() {
        let e1 = SearchLineItem {
            text: "base times height, is one half times the second derivative of the area function, evaluated at".to_string(),
            highlight_ranges: vec1![
                HighlightRange {
                    beginning: 41,
                    end: 47,
                },
            ],
            score: 50,
            internal_annotations: vec![],
            fields: HashMap::default(),
            content_offset: 12658,
            url_suffix: None,
        };

        let e2 = SearchLineItem {

            text: "times height, is one half times the second derivative of the area function, evaluated at a".to_string(),
            highlight_ranges: vec1![
                HighlightRange {
                    beginning: 43,
                    end: 53,
                },
            ],
            score: 50,
            internal_annotations: vec![],
            fields: HashMap::default(),
            content_offset: 12665,
            url_suffix: None,
        };

        let expected = SearchLineItem {
            text: "base times height, is one half times the second derivative of the area function, evaluated at a".to_string(),
            highlight_ranges: vec1![
                HighlightRange {
                    beginning: 41,
                    end: 47,
                },
                HighlightRange {
                    beginning: 48,
                    end: 58,
                },
            ],
            content_offset: 12658,
            score: 99,
            fields: HashMap::default(),
            internal_annotations: vec![],
            url_suffix: None,
        };

        assert_eq!(Some(expected), merge_two_excerpts(&e1, &e2));
    }

    #[test]
    fn it_fails_a_merge_when_results_are_too_far_away() {
        let e1 = SearchLineItem {
    text: "series, I make frequent reference to higher order derivatives. And, if you're already comfortable with second derivatives, third derivatives and such, great! Feel free".to_string(),
    highlight_ranges: vec1![
        HighlightRange {
            beginning: 50,
            end: 61,
        },
        HighlightRange {
            beginning: 105,
            end: 111,
        },
        HighlightRange {
            beginning: 112,
            end: 123,
        },
    ],
    content_offset: 84,
    score: 16,
    fields: HashMap::new(),
    internal_annotations: vec![],
    url_suffix: None,
};
        let e2 = SearchLineItem {
    text: "if you're already comfortable with second derivatives, third derivatives and such, great! Feel free to skip".to_string(),
    highlight_ranges: vec1![
        HighlightRange {
            beginning: 63,
            end: 74,
        },
    ],
    content_offset: 165,
    score: 34,
    fields: HashMap::new(),
    internal_annotations: vec![],
    url_suffix: None,
};

        assert!(merge_two_excerpts(&e1, &e2).is_none());
    }

    #[test]
    fn it_can_merge_two_strings_at_offset_zero() {
        let e1 = SearchLineItem {
            text: "In the next chapter, about Taylor series, I make frequent reference to higher"
                .to_string(),
            highlight_ranges: vec1![HighlightRange {
                beginning: 27,
                end: 33,
            }],
            content_offset: 27,
            score: 38,
            fields: HashMap::new(),
            internal_annotations: vec![],
            url_suffix: None,
        };

        let e2 = SearchLineItem {
    text: "In the next chapter, about Taylor series, I make frequent reference to higher order".to_string(),
    highlight_ranges: vec1![
        HighlightRange {
            beginning: 34,
            end: 40,
        },
    ],
    content_offset: 34,
    score: 50,
    fields: HashMap::new(),
    internal_annotations: vec![],
    url_suffix: None,
};

        let expected = SearchLineItem {
    text: "In the next chapter, about Taylor series, I make frequent reference to higher order".to_string(),
    highlight_ranges: vec1![
        HighlightRange {
                beginning: 27,
                end: 33,
            },
        HighlightRange {
            beginning: 34,
            end: 40,
        },
    ],
    content_offset: 27,
    score: 87,
    fields: HashMap::new(),
    internal_annotations: vec![],
    url_suffix: None,
};

        assert_eq!(Some(expected), merge_two_excerpts(&e1, &e2))
    }
}
