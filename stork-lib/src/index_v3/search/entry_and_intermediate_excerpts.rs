use std::collections::HashMap;

use crate::{
    config::TitleBoost,
    index_v3::{Entry, PassthroughConfig, WordListSource},
    Excerpt, HighlightRange, Result,
};

use super::intermediate_excerpt::IntermediateExcerpt;

#[derive(Debug)]
pub(super) struct EntryAndIntermediateExcerpts {
    pub(super) entry: Entry,
    pub(super) config: PassthroughConfig,
    pub(super) intermediate_excerpts: Vec<IntermediateExcerpt>,
}

#[allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
impl From<EntryAndIntermediateExcerpts> for Result {
    fn from(data: EntryAndIntermediateExcerpts) -> Self {
        let entry = data.entry;
        let excerpt_buffer = data.config.excerpt_buffer as usize;

        let split_contents: Vec<String> = entry
            .contents
            .split_whitespace()
            .map(ToString::to_string)
            .collect();

        let mut ies: Vec<&IntermediateExcerpt> = data
            .intermediate_excerpts
            .iter()
            .filter(|ie| ie.source == WordListSource::Contents)
            .collect();

        // Get rid of intermediate excerpts that refer to the same word index.
        // But first, sort by score so that only the highest score within the
        // same word index is kept.
        ies.sort_by_cached_key(|ie| ie.score);
        ies.sort_by_cached_key(|ie| ie.word_index);
        ies.dedup_by_key(|ie| ie.word_index);

        let mut ies_grouped_by_word_index: Vec<Vec<&IntermediateExcerpt>> = vec![];

        for ie in &ies {
            if let Some(most_recent) = ies_grouped_by_word_index.last_mut() {
                if let Some(trailing_ie) = most_recent.first() {
                    if (ie.word_index as isize) - (trailing_ie.word_index as isize)
                        < (excerpt_buffer as isize)
                    {
                        most_recent.push(ie);
                        continue;
                    }
                }
            }

            ies_grouped_by_word_index.push(vec![ie])
        }

        let mut excerpts: Vec<Excerpt> = ies_grouped_by_word_index
            .iter()
            .map(|ies| {
                let minimum_word_index = ies
                    .first()
                    .unwrap()
                    .word_index
                    .saturating_sub(excerpt_buffer);

                let maximum_word_index = std::cmp::min(
                    ies.last()
                        .unwrap()
                        .word_index
                        .saturating_add(excerpt_buffer),
                    split_contents.len(),
                );

                let text = split_contents[minimum_word_index..maximum_word_index].join(" ");

                let mut highlight_ranges: Vec<HighlightRange> = ies
                    .iter()
                    .map(|ie| {
                        let beginning = split_contents[minimum_word_index..ie.word_index]
                            .join(" ")
                            .chars()
                            .count()
                            + 1;
                        HighlightRange {
                            beginning,
                            end: beginning + ie.query.chars().count(),
                        }
                    })
                    .collect();

                highlight_ranges.sort();

                let highlighted_character_range = highlight_ranges.last().unwrap().end
                    - highlight_ranges.first().unwrap().beginning;

                let highlighted_characters_count: usize = highlight_ranges
                    .iter()
                    .map(|hr| hr.end - hr.beginning)
                    .sum();

                let score_modifier = highlighted_character_range - highlighted_characters_count;

                let score = ies
                    .iter()
                    .map(|ie| (ie.score as usize))
                    .sum::<usize>()
                    .saturating_sub(score_modifier);

                // Since we're mapping from multiple IntermediateExcerpts to one
                // Excerpt, we have to either combine or filter data. For
                // `fields` and `internal_annotations`, I'm taking the data from
                // the first intermediate excerpt in the vector.
                let fields = ies
                    .first()
                    .map_or_else(HashMap::new, |first| first.fields.clone());

                let internal_annotations = ies
                    .first()
                    .map_or_else(Vec::default, |first| first.internal_annotations.clone());

                Excerpt {
                    text,
                    highlight_ranges,
                    score,
                    internal_annotations,
                    fields,
                }
            })
            .collect();

        excerpts.sort_by_key(|e| -(e.score as i16));
        excerpts.truncate(data.config.excerpts_per_result as usize);

        let split_title: Vec<&str> = entry
            .title
            .split(|c: char| c.is_ascii_whitespace() || c == '-')
            .collect();
        let mut title_highlight_ranges: Vec<HighlightRange> = data
            .intermediate_excerpts
            .iter()
            .filter(|&ie| ie.source == WordListSource::Title)
            .map(|ie| {
                let space_offset = if ie.word_index == 0 { 0 } else { 1 };
                let beginning = split_title[0..ie.word_index].join(" ").len() + space_offset;
                HighlightRange {
                    beginning,
                    end: beginning + ie.query.len(),
                }
            })
            .collect();

        title_highlight_ranges.sort();

        let title_boost_modifier = title_highlight_ranges.len()
            * match data.config.title_boost {
                TitleBoost::Minimal => 25,
                TitleBoost::Moderate => 75,
                TitleBoost::Large => 150,
                TitleBoost::Ridiculous => 5000,
            };

        // Sort each result by a sum of an exponental backoff of its excerpts' scores.
        // This more evenly weights a single high score excerpt vs multiple low score excerpts.
        let result_score = {
            let mut sorted_excerpt_scores: Vec<usize> = excerpts.iter().map(|e| e.score).collect();
            sorted_excerpt_scores.sort();

            let sum: usize = sorted_excerpt_scores
                .into_iter()
                .enumerate()
                .map(|(index, score)| {
                    let score = score as f32;
                    let index = index as f32;
                    let excerpt_score = score * 2f32.powf(1f32 / (index + 1f32)); // index + 1 to avoid ÷0 errors
                    excerpt_score as usize
                })
                .sum();

            sum + title_boost_modifier
        };

        Result {
            entry: crate::Entry::from(entry),
            excerpts,
            title_highlight_ranges,
            score: result_score,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::index_v3::Entry;
    use pretty_assertions::assert_eq;

    #[test]
    fn result_with_multiple_excerpts_sorts_higher_than_result_with_single() {
        let multiple_excerpts = EntryAndIntermediateExcerpts {
            entry: Entry {
                contents: "".to_string(),
                title: "The quick brown fox jumps over the lazy dog".to_string(),
                url: String::default(),
                fields: HashMap::default(),
            },
            config: PassthroughConfig::default(),
            intermediate_excerpts: vec![
                IntermediateExcerpt {
                    query: "over".to_string(),
                    entry_index: 0,
                    score: 128,
                    source: WordListSource::Title,
                    word_index: 3,
                    internal_annotations: Vec::default(),
                    fields: HashMap::default(),
                },
                IntermediateExcerpt {
                    query: "brown".to_string(),
                    entry_index: 0,
                    score: 128,
                    source: WordListSource::Title,
                    word_index: 2,
                    internal_annotations: Vec::default(),
                    fields: HashMap::default(),
                },
            ],
        };

        let single_excerpt = EntryAndIntermediateExcerpts {
            entry: Entry {
                contents: "".to_string(),
                title: "The quick brown fox jumps over the lazy dog".to_string(),
                url: String::default(),
                fields: HashMap::default(),
            },
            config: PassthroughConfig::default(),
            intermediate_excerpts: vec![IntermediateExcerpt {
                query: "over".to_string(),
                entry_index: 0,
                score: 128,
                source: WordListSource::Title,
                word_index: 3,
                internal_annotations: Vec::default(),
                fields: HashMap::default(),
            }],
        };

        let result_multiple_excerpts = Result::from(multiple_excerpts);
        assert_eq!(result_multiple_excerpts.score, 150);

        let result_single_excerpt = Result::from(single_excerpt);
        assert_eq!(result_single_excerpt.score, 75);
    }

    #[test]
    fn title_highlight_ranges_are_sorted() {
        let entry_and_intermediate_excerpts = EntryAndIntermediateExcerpts {
            entry: Entry {
                contents: "".to_string(),
                title: "The quick brown fox jumps over the lazy dog".to_string(),
                url: String::default(),
                fields: HashMap::default(),
            },
            config: PassthroughConfig::default(),
            intermediate_excerpts: vec![
                IntermediateExcerpt {
                    query: "over".to_string(),
                    entry_index: 0,
                    score: 128,
                    source: WordListSource::Title,
                    word_index: 3,
                    internal_annotations: Vec::default(),
                    fields: HashMap::default(),
                },
                IntermediateExcerpt {
                    query: "brown".to_string(),
                    entry_index: 0,
                    score: 128,
                    source: WordListSource::Title,
                    word_index: 2,
                    internal_annotations: Vec::default(),
                    fields: HashMap::default(),
                },
            ],
        };
        let output_result = Result::from(entry_and_intermediate_excerpts);
        let title_highlight_ranges = output_result.title_highlight_ranges;
        println!("{:?}", title_highlight_ranges);
        assert!(
            title_highlight_ranges[0].beginning <= title_highlight_ranges[1].beginning,
            "Title highlight ranges were not sorted! [0].beginning is {} while [1].beginning is {}",
            title_highlight_ranges[0].beginning,
            title_highlight_ranges[1].beginning
        );
    }

    #[test]
    fn highlighting_does_not_offset_with_special_characters() {
        let entry_and_intermediate_excerpts = EntryAndIntermediateExcerpts {
            entry: Entry {
                contents: "AFTER a \u{2018}surprisingly\u{2019} unequivocal experience of the inefficiency of the subsisting federal government".to_string(),
                title: "Introduction".to_string(),
                url: String::default(),
                fields: HashMap::default(),
            },
            config: PassthroughConfig::default(),
            intermediate_excerpts: vec![
                IntermediateExcerpt {
                    query: "unequivocal".to_string(),
                    entry_index: 0,
                    score: 128,
                    source: WordListSource::Contents,
                    word_index: 3,
                    internal_annotations: Vec::default(),
                    fields: HashMap::default(),
                },
                IntermediateExcerpt {
                    query: "\u{2018}surprisingly\u{2019}".to_string(),
                    entry_index: 0,
                    score: 128,
                    source: WordListSource::Contents,
                    word_index: 2,
                    internal_annotations: Vec::default(),
                    fields: HashMap::default(),
                },
            ],
        };

        let output_result = Result::from(entry_and_intermediate_excerpts);
        let excerpt = output_result.excerpts.first().unwrap();
        let excerpt_chars = excerpt.text.chars().collect::<Vec<char>>();
        let first_highlight_range = &excerpt.highlight_ranges.first().unwrap();
        let computed_first_word = &excerpt_chars
            [first_highlight_range.beginning..first_highlight_range.end]
            .iter()
            .collect::<String>();
        let second_highlight_range = &excerpt.highlight_ranges[1];
        let computed_second_word = &excerpt_chars
            [second_highlight_range.beginning..second_highlight_range.end]
            .iter()
            .collect::<String>();

        assert_eq!(
            computed_second_word, "unequivocal",
            "Expected `unequivocal`, got {}",
            computed_second_word
        );

        assert_eq!(
            computed_first_word, "\u{2018}surprisingly\u{2019}",
            "Expected `\u{2018}surprisingly\u{2019}`, got {}",
            computed_first_word
        );
    }

    #[test]
    #[ignore = "No assertion in test"]
    fn title_highlighting_works_when_title_has_no_spaces() {
        let entry_and_intermediate_excerpts = EntryAndIntermediateExcerpts {
            entry: Entry {
                contents: "".to_string(),
                title: "api-methods-animate".to_string(),
                url: String::default(),
                fields: HashMap::default(),
            },
            config: PassthroughConfig::default(),
            intermediate_excerpts: vec![IntermediateExcerpt {
                query: "anim".to_string(),
                entry_index: 0,
                score: 128,
                source: WordListSource::Title,
                word_index: 2,
                internal_annotations: Vec::default(),
                fields: HashMap::default(),
            }],
        };

        let output_result = Result::from(entry_and_intermediate_excerpts);

        dbg!(output_result);
    }
}
