use super::stopwords::STOPWORDS;
use super::structs::*;
use crate::searcher::*;
use crate::IndexFromFile;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::convert::TryInto;

fn deserialize(index: &IndexFromFile) -> Index {
    let (version_size_bytes, rest) = index.split_at(std::mem::size_of::<u64>());
    let version_size = u64::from_be_bytes(version_size_bytes.try_into().unwrap());
    let (_version_bytes, rest) = rest.split_at(version_size as usize);

    let (entries_size_bytes, rest) = rest.split_at(std::mem::size_of::<u64>());
    let entries_size = u64::from_be_bytes(entries_size_bytes.try_into().unwrap());
    let (entries_bytes, rest) = rest.split_at(entries_size as usize);
    let entries = bincode::deserialize(entries_bytes).unwrap();

    let (queries_size_bytes, rest) = rest.split_at(std::mem::size_of::<u64>());
    let queries_size = u64::from_be_bytes(queries_size_bytes.try_into().unwrap());
    let (queries_bytes, _rest) = rest.split_at(queries_size as usize);
    let queries = bincode::deserialize(queries_bytes).unwrap();

    Index { entries, queries }
}

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
    fn new(container: Container, query: &String) -> Self {
        ContainerWithQuery {
            query: query.to_string(),
            results: container.results,
            aliases: container.aliases,
        }
    }

    fn get_intermediate_excerpts(&self, index: &Index) -> Vec<IntermediateExcerpt> {
        let mut output = vec![];
        // Put container's results in output
        for (entry_index, result) in self.results.iter() {
            for excerpt in result.excerpts.to_owned() {
                output.push(IntermediateExcerpt {
                    query: self.query.to_string(),
                    entry_index: *entry_index,
                    score: result.score,
                    word_index: excerpt.word_index,
                })
            }
        }

        // Put alias containers' results in output
        for (alias_target, alias_score) in self.aliases.iter() {
            if let Some(target_container) = index.queries.get(alias_target) {
                for (entry_index, result) in target_container.results.to_owned() {
                    for excerpt in result.excerpts.to_owned() {
                        output.push(IntermediateExcerpt {
                            query: alias_target.to_string(),
                            entry_index: entry_index,
                            score: *alias_score,
                            word_index: excerpt.word_index,
                        })
                    }
                }
            }
        }

        output
    }
}

impl OutputEntry {
    fn from(entry: &Entry) -> Self {
        OutputEntry {
            url: entry.url.clone(),
            title: entry.title.clone(),
            fields: entry.fields.clone(),
        }
    }
}

impl OutputResult {
    fn from(entry: &Entry, intermediate_excerpts: &Vec<IntermediateExcerpt>) -> Self {
        let split_contents: Vec<String> = entry
            .contents
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();
        let mut ies = intermediate_excerpts.to_vec();
        // Get rid of intermediate excerpts that refer to the same word index.
        // Ideally we'd have sorted by score within the same word index so that
        // only the highest score is kept.
        ies.sort_by_key(|ie| ie.word_index);
        ies.dedup_by_key(|ie| ie.word_index);

        // println!("{} {:#?}\n{}\n", entry.title, ies, "=".repeat(32 * 8));

        let mut ies_grouped_by_word_index: Vec<Vec<&IntermediateExcerpt>> = vec![];

        for ie in &ies {
            if let Some(most_recent) = ies_grouped_by_word_index.last_mut() {
                if let Some(trailing_ie) = most_recent.first() {
                    // println!("{}/{}", trailing_ie.word_index, ie.word_index);
                    if (ie.word_index as isize) - (trailing_ie.word_index as isize) < 8 {
                        most_recent.push(ie);
                        continue;
                    }
                }
            }

            ies_grouped_by_word_index.push(vec![ie])
        }

        // println!("{} {:#?}\n{}\n", entry.title, ies_grouped_by_word_index, "=".repeat(32 * 8));

        let mut excerpts: Vec<crate::searcher::Excerpt> = ies_grouped_by_word_index
            .iter()
            .map(|ies| {
                let minimum_word_index = ies.first().unwrap().word_index.saturating_sub(8);

                let maximum_word_index = std::cmp::min(
                    ies.last().unwrap().word_index.saturating_add(8),
                    split_contents.len(),
                );

                let text = split_contents[minimum_word_index..maximum_word_index].join(" ");

                let highlight_ranges: Vec<HighlightRange> = ies
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

                let score = ies.iter().map(|ie| (ie.score as usize)).sum();

                crate::searcher::Excerpt {
                    text,
                    highlight_ranges,
                    score,
                }
            })
            .collect();
        

        excerpts.sort_by_key(|e| (e.score as i16) * -1);
        excerpts.truncate(10);

        let mut score: usize = excerpts.iter().map(|e| (e.score as usize)).sum();
        if excerpts.len() > 3 {
            let sum_of_first_three = &excerpts[0..3].iter().map(|e| (e.score as usize)).sum();
            let mean_of_rest = &excerpts[3..].iter().map(|e| (e.score as usize)).sum() / (excerpts.len() - 3);
            score = sum_of_first_three + mean_of_rest;
        }

        // @TODO check for index out of bounds errors

        return OutputResult {
            entry: OutputEntry::from(entry),
            excerpts,
            title_highlight_char_offset: None,
            score,
        };
    }
}

pub fn search(index: &IndexFromFile, query: &str) -> SearchOutput {
    let index: Index = deserialize(index);
    let normalized_query = query.to_lowercase();
    let words_in_query: Vec<String> = normalized_query.split(' ').map(|s| s.to_string()).collect();

    // Get containers for each word in the query
    let mut intermediate_excerpts: Vec<IntermediateExcerpt> = words_in_query
        .iter()
        .flat_map(|word| index.queries.get_key_value(word))
        .map(|(word, ctr)| ContainerWithQuery::new(ctr.to_owned(), word))
        .map(|ctr_query| ctr_query.get_intermediate_excerpts(&index))
        .flatten()
        .collect();

    for mut ie in &mut intermediate_excerpts {
        if STOPWORDS.contains(&ie.query.as_str()) {
            ie.score = 40;
        }
    }

    let mut excerpts_by_index: HashMap<EntryIndex, Vec<IntermediateExcerpt>> = HashMap::new();
    for ie in intermediate_excerpts {
        excerpts_by_index
            .entry(ie.entry_index)
            .or_insert(vec![])
            .push(ie)
    }

    let mut output_results: Vec<OutputResult> = excerpts_by_index
        .iter()
        .map(|(entry_index, ie)| OutputResult::from(&index.entries[*entry_index], &ie))
        .take(10)
        .collect();
    output_results.sort_by_key(|or| (or.score as i64) * -1);
    let or_len = &output_results.len();

    SearchOutput {
        results: output_results,
        total_hit_count: *or_len,
    }
}