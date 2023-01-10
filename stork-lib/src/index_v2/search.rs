use std::cmp::Ordering;
use std::collections::HashMap;

use crate::{stopwords, Excerpt, HighlightRange, Output, Result};

use super::{
    scores::STOPWORD_SCORE, AliasTarget, Container, Entry, EntryIndex, Index, Score, SearchResult,
};

const EXCERPT_BUFFER: usize = 8;
const EXCERPTS_PER_RESULT: usize = 5;
const DISPLAYED_RESULTS_COUNT: usize = 10;

#[derive(Clone, Debug, Eq)]
struct IntermediateExcerpt {
    query: String,
    entry_index: EntryIndex,
    score: Score,
    word_index: usize,
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

impl PartialEq for IntermediateExcerpt {
    fn eq(&self, other: &Self) -> bool {
        self.entry_index == other.entry_index
    }
}

struct ContainerWithQuery {
    results: HashMap<EntryIndex, SearchResult>,
    aliases: HashMap<AliasTarget, Score>,
    query: String,
}

impl ContainerWithQuery {
    fn new(container: Container, query: &str) -> Self {
        ContainerWithQuery {
            query: query.to_string(),
            results: container.results,
            aliases: container.aliases,
        }
    }

    fn get_intermediate_excerpts(&self, index: &Index) -> Vec<IntermediateExcerpt> {
        let mut output = vec![];
        // Put container's results in output
        for (entry_index, result) in &self.results {
            for excerpt in result.excerpts.clone() {
                output.push(IntermediateExcerpt {
                    query: self.query.to_string(),
                    entry_index: *entry_index,
                    score: result.score,
                    word_index: excerpt.word_index,
                });
            }
        }

        // Put alias containers' results in output
        for (alias_target, alias_score) in &self.aliases {
            if let Some(target_container) = index.queries.get(alias_target) {
                for (entry_index, result) in target_container.results.clone() {
                    for excerpt in result.excerpts.clone() {
                        output.push(IntermediateExcerpt {
                            query: alias_target.to_string(),
                            entry_index,
                            score: *alias_score,
                            word_index: excerpt.word_index,
                        });
                    }
                }
            }
        }

        output
    }
}

impl From<crate::index_v2::Entry> for crate::Entry {
    fn from(entry: crate::index_v2::Entry) -> Self {
        crate::Entry {
            url: entry.url.clone(),
            title: entry.title.clone(),
            fields: entry.fields.unwrap_or_default(),
        }
    }
}

struct EntryAndIntermediateExcerpts {
    entry: Entry,
    intermediate_excerpts: Vec<IntermediateExcerpt>,
}

impl From<EntryAndIntermediateExcerpts> for Result {
    fn from(data: EntryAndIntermediateExcerpts) -> Self {
        let entry = data.entry;
        let mut ies = data.intermediate_excerpts;
        let split_contents: Vec<String> = entry
            .contents
            .split_whitespace()
            .map(std::string::ToString::to_string)
            .collect();

        // Get rid of intermediate excerpts that refer to the same word index.
        // First, sort by score so that only the highest score within the same
        // word index is kept.
        ies.sort_by_cached_key(|ie| ie.score);
        ies.sort_by_cached_key(|ie| ie.word_index);
        ies.dedup_by_key(|ie| ie.word_index);

        let mut ies_grouped_by_word_index: Vec<Vec<&IntermediateExcerpt>> = vec![];

        for ie in &ies {
            if let Some(most_recent) = ies_grouped_by_word_index.last_mut() {
                if let Some(trailing_ie) = most_recent.first() {
                    if ie.word_index - trailing_ie.word_index < EXCERPT_BUFFER {
                        most_recent.push(ie);
                        continue;
                    }
                }
            }

            ies_grouped_by_word_index.push(vec![ie]);
        }

        let mut excerpts: Vec<Excerpt> = ies_grouped_by_word_index
            .iter()
            .map(|ies| {
                let minimum_word_index = ies
                    .first()
                    .unwrap()
                    .word_index
                    .saturating_sub(EXCERPT_BUFFER);

                let maximum_word_index = std::cmp::min(
                    ies.last()
                        .unwrap()
                        .word_index
                        .saturating_add(EXCERPT_BUFFER),
                    split_contents.len(),
                );

                let text = split_contents[minimum_word_index..maximum_word_index].join(" ");

                let mut highlight_ranges: Vec<HighlightRange> = ies
                    .iter()
                    .map(|ie| {
                        let beginning = split_contents[minimum_word_index..ie.word_index]
                            .join(" ")
                            .len()
                            + 1;
                        HighlightRange {
                            beginning,
                            end: beginning + ie.query.len(),
                        }
                    })
                    .collect();
                // Maybe unneccesary?
                highlight_ranges.sort_by_key(|hr| hr.beginning);

                let highlighted_character_range = highlight_ranges.last().unwrap().end
                    - highlight_ranges.first().unwrap().beginning;

                let highlighted_characters_count: usize = highlight_ranges
                    .iter()
                    .map(|hr| hr.end - hr.beginning)
                    .sum();

                let score_modifier = highlighted_character_range - highlighted_characters_count;

                let score =
                    ies.iter().map(|ie| (ie.score as usize)).sum::<usize>() - score_modifier;

                Excerpt {
                    text,
                    highlight_ranges,
                    score,
                    internal_annotations: Vec::default(),
                    fields: HashMap::default(),
                }
            })
            .collect();

        excerpts.sort_by(|a, b| b.score.cmp(&a.score));
        excerpts.truncate(EXCERPTS_PER_RESULT);

        let score = excerpts.first().map_or(0, |first| first.score);

        Result {
            entry: crate::Entry::from(entry),
            excerpts,
            title_highlight_ranges: vec![],
            score,
        }
    }
}

pub fn search(index: &Index, query: &str) -> Output {
    let normalized_query = query.to_lowercase();
    let words_in_query: Vec<String> = normalized_query
        .split(' ')
        .map(std::string::ToString::to_string)
        .collect();

    // Get containers for each word in the query
    let mut intermediate_excerpts: Vec<IntermediateExcerpt> = words_in_query
        .iter()
        .filter_map(|word| index.queries.get_key_value(word))
        .map(|(word, ctr)| ContainerWithQuery::new(ctr.clone(), word))
        .flat_map(|ctr_query| ctr_query.get_intermediate_excerpts(index))
        .collect();

    for mut ie in &mut intermediate_excerpts {
        if stopwords.contains(&ie.query.as_str()) {
            ie.score = STOPWORD_SCORE;
        }
    }

    let mut excerpts_by_index: HashMap<EntryIndex, Vec<IntermediateExcerpt>> = HashMap::new();
    for ie in intermediate_excerpts {
        excerpts_by_index
            .entry(ie.entry_index)
            .or_insert_with(Vec::new)
            .push(ie);
    }

    let total_len = &excerpts_by_index.len();

    let mut output_results: Vec<Result> = excerpts_by_index
        .iter()
        .map(|(entry_index, ies)| {
            let data = EntryAndIntermediateExcerpts {
                entry: index.entries[*entry_index].clone(),
                intermediate_excerpts: ies.clone(),
            };
            Result::from(data)
        })
        .collect();
    output_results.sort_by_key(|or| or.entry.title.clone());
    output_results.sort_by_key(|or| -(or.score as i64));
    output_results.truncate(DISPLAYED_RESULTS_COUNT);

    Output {
        results: output_results,
        total_hit_count: *total_len,
        url_prefix: String::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::{BufReader, Read};

    #[test]
    fn e2e_v2_search_works() {
        let file = fs::File::open("../test-assets/federalist-min-0.6.0.st").unwrap();
        let mut buf_reader = BufReader::new(file);
        let mut index_bytes: Vec<u8> = Vec::new();
        let _bytes_read = buf_reader.read_to_end(&mut index_bytes);

        let index = Index::from_file(index_bytes.as_slice());
        let generated = search(&index, "liber old world");
        let expected = serde_json::from_str("{\"results\":[{\"entry\":{\"url\":\"https://www.congress.gov/resources/display/content/The+Federalist+Papers#TheFederalistPapers-1\",\"title\":\"Introduction\",\"fields\":{}},\"excerpts\":[{\"text\":\"in many respects the most interesting in the world. It has been frequently remarked that it\",\"highlight_ranges\":[{\"beginning\":45,\"end\":50}],\"score\":128,\"internal_annotations\":[],\"fields\":{}},{\"text\":\"despotic power and hostile to the principles of liberty. An over-scrupulous jealousy of danger to the\",\"highlight_ranges\":[{\"beginning\":48,\"end\":55}],\"score\":125,\"internal_annotations\":[],\"fields\":{}},{\"text\":\"of love, and that the noble enthusiasm of liberty is apt to be infected with a\",\"highlight_ranges\":[{\"beginning\":42,\"end\":49}],\"score\":125,\"internal_annotations\":[],\"fields\":{}},{\"text\":\"of government is essential to the security of liberty; that, in the contemplation of a sound\",\"highlight_ranges\":[{\"beginning\":46,\"end\":53}],\"score\":125,\"internal_annotations\":[],\"fields\":{}},{\"text\":\"that this is the safest course for your liberty, your dignity, and your happiness. I affect\",\"highlight_ranges\":[{\"beginning\":40,\"end\":47}],\"score\":125,\"internal_annotations\":[],\"fields\":{}}],\"title_highlight_ranges\":[],\"score\":128}],\"total_hit_count\":1,\"url_prefix\":\"\"}").unwrap();

        // assert_eq!(
        //     IndexVersion::from(ParsedIndex::try_from(index_bytes.as_slice()).unwrap()),
        //     IndexVersion::V2
        // );
        assert_eq!(generated, expected, "{generated:?}");
    }
}
