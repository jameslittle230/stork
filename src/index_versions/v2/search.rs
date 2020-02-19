use super::structs::*;
use crate::searcher::*;
use crate::IndexFromFile;
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

impl OutputResult {
    fn from(entry: &Entry, results: Vec<SearchResult>) -> OutputResult {
        let entry_contents_by_word: Vec<String> = entry
            .contents
            .split_whitespace()
            .map(|w| w.to_string())
            .collect();
        let entry_contents_by_word_len = entry_contents_by_word.len();

        OutputResult {
            entry: OutputEntry {
                title: entry.title.clone(),
                url: entry.url.clone(),
                fields: entry.fields.clone(),
            },
            excerpts: results
                .iter()
                // sort by score
                .map(|result| &result.excerpts)
                .flatten()
                .map(|excerpt| {
                    crate::searcher::Excerpt {
                        text: entry_contents_by_word[excerpt.word_index.saturating_sub(8)
                            ..std::cmp::min(
                                excerpt.word_index.saturating_add(8),
                                entry_contents_by_word_len,
                            )]
                            .join(" "),
                        highlight_char_offset: entry_contents_by_word
                            [excerpt.word_index.saturating_sub(8)..excerpt.word_index]
                            .join(" ")
                            .len()
                            + 1,
                    }
                })
                .collect(),
            title_highlight_char_offset: None,
        }
    }
}

pub fn search(index: &IndexFromFile, query: &str) -> SearchOutput {
    let index: Index = deserialize(index);
    let normalized_query = query.to_lowercase();
    let words_in_query: Vec<String> = normalized_query.split(' ').map(|s| s.to_string()).collect();
    let first_word = &words_in_query[0];
    let container: Container = index.queries.get(first_word).unwrap().to_owned();

    #[derive(Debug)]
    struct ScoreToContainer {
        score: Score,
        container: Container,
    }

    let mut aliased_results = container
        .aliases
        .keys()
        .map(|alias| ScoreToContainer {
            score: container.aliases.get(alias).unwrap().to_owned(),
            container: index.queries.get(alias).unwrap().to_owned(),
        })
        .map(|score_to_container| {
            let mut results = score_to_container.container.results;
            for result in results.values_mut() {
                result.score = score_to_container.score
            }
            results
        })
        .collect::<Vec<HashMap<EntryIndex, SearchResult>>>();
    let mut all_results: Vec<HashMap<EntryIndex, SearchResult>> = vec![];
    if !container.results.is_empty() {
        all_results.append(&mut vec![container.results])
    }
    if !aliased_results.is_empty() {
        all_results.append(&mut aliased_results);
    }

    let mut combined_results: HashMap<EntryIndex, Vec<SearchResult>> = HashMap::new();

    for result_map in &all_results {
        for (index, result) in result_map.iter() {
            let result_vec = combined_results.entry(*index).or_insert_with(Vec::new);
            result_vec.push(result.to_owned());
        }
    }

    let output_results: Vec<OutputResult> = combined_results
        .iter()
        .map(|(entry_index, results)| {
            let entry = &index.entries[*entry_index];
            OutputResult::from(entry, results.to_vec())
        })
        .collect();

    SearchOutput {
        results: output_results[0..std::cmp::min(output_results.len(), 10)].to_vec(),
        total_hit_count: output_results.len(),
    }
}
