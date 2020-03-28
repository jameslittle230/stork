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
    fn from(entry: &Entry, result: &SearchResultWithAnnotatedExcerpts) -> OutputResult {
        let entry_contents_by_word: Vec<String> = entry
            .contents
            .split_whitespace()
            .map(|w| w.to_string())
            .collect();
        let entry_contents_by_word_len = entry_contents_by_word.len();

        let mut output_excerpts: Vec<crate::searcher::Excerpt> = Vec::new();

        let mut input_excerpts = result.excerpts.clone();
        input_excerpts.sort_by(|a, b| a.excerpt.word_index.cmp(&b.excerpt.word_index));

        #[derive(Debug)]
        struct QueryWordToWordIndex {
            word_index: usize,
            query: String,
        }

        #[derive(Debug)]
        struct CombinedExcerpt {
            word_indices: Vec<QueryWordToWordIndex>,
        }

        let mut combined_excerpts: Vec<CombinedExcerpt> = vec![];

        // this seems... dangerous
        let mut idx = 0;
        while idx < input_excerpts.len() {
            let rabbit = input_excerpts.get(idx).unwrap();
            combined_excerpts.push(CombinedExcerpt {
                word_indices: vec![QueryWordToWordIndex {
                    word_index: rabbit.excerpt.word_index,
                    query: rabbit.word.to_string(),
                }],
            });

            let mut hare_index = idx;
            while let Some(hare) = input_excerpts.get(hare_index + 1) {
                if hare.excerpt.word_index - rabbit.excerpt.word_index < 8 {
                    combined_excerpts
                        .last_mut()
                        .unwrap()
                        .word_indices
                        .push(QueryWordToWordIndex {
                            word_index: hare.excerpt.word_index,
                            query: hare.word.to_string(),
                        });
                    // .push(hare.word_index);
                    hare_index += 1;
                } else {
                    idx = hare_index;
                    break;
                }
            }
            idx += 1
        }

        println!("{:#?}", combined_excerpts);

        for excerpt in combined_excerpts {
            let mut highlight_ranges = excerpt
                .word_indices
                .iter()
                .map(|query_word_index_struct| {
                    let minimum_word_index = excerpt
                        .word_indices
                        .first()
                        .unwrap()
                        .word_index
                        .saturating_sub(8);

                    let maximum_word_index = query_word_index_struct.word_index;

                    let beginning = entry_contents_by_word[minimum_word_index..maximum_word_index]
                        .join(" ")
                        .len()
                        + 1;
                    HighlightRange {
                        beginning,
                        end: beginning + query_word_index_struct.query.len(),
                    }
                })
                .collect::<Vec<HighlightRange>>();

            highlight_ranges.sort_by(|a, b| a.beginning.cmp(&b.beginning));

            let minimum_word_index = excerpt
                .word_indices
                .first()
                .unwrap()
                .word_index
                .saturating_sub(8);

            let maximum_word_index = std::cmp::min(
                excerpt
                    .word_indices
                    .last()
                    .unwrap()
                    .word_index
                    .saturating_add(8),
                entry_contents_by_word_len,
            );
            
            let e = crate::searcher::Excerpt {
                text: entry_contents_by_word[minimum_word_index..maximum_word_index].join(" "),
                highlight_ranges,
                score: (result.score as usize) * excerpt.word_indices.len(),
            };

            output_excerpts.push(e);
        }

        output_excerpts.sort_by(|a, b| b.score.cmp(&a.score));

        let score = output_excerpts
            .iter()
            .map(|e| e.score)
            .max()
            .unwrap_or(0)
            .saturating_mul(output_excerpts.len());

        OutputResult {
            entry: OutputEntry {
                title: entry.title.clone(),
                url: entry.url.clone(),
                fields: entry.fields.clone(),
            },
            score,
            excerpts: output_excerpts,
            title_highlight_char_offset: None,
        }
    }
}

#[derive(Debug)]
struct ScoreToContainer {
    score: Score,
    container: Container,
}

#[derive(Debug, Clone)]
struct QueryWordToContainer {
    word: String,
    container: Container,
}

#[derive(Debug, Clone)]
struct QueryWordToSearchResult {
    word: String,
    result: SearchResult,
}

#[derive(Debug, Clone)]
pub struct QueryWordToExcerpt {
    word: String,
    excerpt: super::structs::Excerpt,
}

pub(super) struct SearchResultWithAnnotatedExcerpts {
    pub(super) excerpts: Vec<QueryWordToExcerpt>,
    pub(super) score: Score,
}

fn resolve_aliases(
    index: &Index,
    query: String,
    container: &Container,
) -> Vec<HashMap<EntryIndex, QueryWordToSearchResult>> {
    // Get results hash maps for _just_ the container's aliases
    let aliased_results = container
        .aliases
        .keys()
        .map(|alias_target| ScoreToContainer {
            score: container.aliases.get(alias_target).unwrap().to_owned(),
            container: index.queries.get(alias_target).unwrap().to_owned(),
        })
        .map(|score_to_container| {
            let mut results = score_to_container.container.results;
            for result in results.values_mut() {
                result.score = score_to_container.score
            }
            results
        })
        .collect::<Vec<HashMap<EntryIndex, SearchResult>>>();

    // Combine the container's results hashes with the aliased results hash to get
    // a list of result hashmaps.
    //
    // We might have the same EntryIndex spread multiple times throughout the
    // vector, each pointing to a different SearchResult.
    let mut all_results: Vec<HashMap<EntryIndex, QueryWordToSearchResult>> = vec![];
    if !container.results.is_empty() {
        let mut results_mapped_to_structs: HashMap<EntryIndex, QueryWordToSearchResult> =
            HashMap::new();
        for (key, val) in container.results.iter() {
            results_mapped_to_structs.insert(
                *key,
                QueryWordToSearchResult {
                    word: query.to_string(),
                    result: val.to_owned(),
                },
            );
        }
        all_results.push(results_mapped_to_structs);
    }

    for aliased_result in aliased_results {
        let mut results_mapped_to_structs: HashMap<EntryIndex, QueryWordToSearchResult> =
            HashMap::new();
        for (key, val) in aliased_result.iter() {
            results_mapped_to_structs.insert(
                *key,
                QueryWordToSearchResult {
                    word: query.to_string(),
                    result: val.to_owned(),
                },
            );
        }
        all_results.push(results_mapped_to_structs);
    }

    all_results
}

pub fn search(index: &IndexFromFile, query: &str) -> SearchOutput {
    let index: Index = deserialize(index);
    let normalized_query = query.to_lowercase();
    let words_in_query: Vec<String> = normalized_query.split(' ').map(|s| s.to_string()).collect();

    // Get containers for each word in the query
    let containers: Vec<QueryWordToContainer> = words_in_query
        .iter()
        .flat_map(|word| {
            if let Some(ctr) = index.queries.get(word) {
                Some(QueryWordToContainer {
                    word: word.to_string(),
                    container: ctr.to_owned(),
                })
            } else {
                return None;
            }
        })
        .collect();
    // Resolve aliases for those containers so we get a bunch of result hashes
    let results_with_duplicates: Vec<HashMap<EntryIndex, QueryWordToSearchResult>> = containers
        .iter()
        .map(|ctr| resolve_aliases(&index, ctr.word.to_string(), &ctr.container))
        .flatten()
        .collect::<Vec<HashMap<EntryIndex, QueryWordToSearchResult>>>();

    // Turn the vector of hashmaps into a single hashmap, joining the results
    // on the entry index.
    let mut combined_results: HashMap<EntryIndex, Vec<QueryWordToSearchResult>> = HashMap::new();
    for result_map in &results_with_duplicates {
        for (index, result) in result_map.iter() {
            let result_vec = combined_results.entry(*index).or_insert_with(Vec::new);
            result_vec.push(result.clone());
        }
    }

    // For each entry in the hash map, turn the vector of results into a single
    // result with concatenated excerpt vectors and summed scores
    let mut combined_excerpts: HashMap<EntryIndex, SearchResultWithAnnotatedExcerpts> =
        HashMap::new();
    for (k, v) in combined_results.iter() {
        for r in v {
            let mut search_result =
                combined_excerpts
                    .entry(k.clone())
                    .or_insert(SearchResultWithAnnotatedExcerpts {
                        excerpts: vec![],
                        score: 0,
                    });

            let mut annotated_excerpts: Vec<QueryWordToExcerpt> = r
                .result
                .excerpts
                .iter()
                .map(|excerpt| QueryWordToExcerpt {
                    word: r.word.to_string(),
                    excerpt: excerpt.to_owned(),
                })
                .collect();
            search_result.excerpts.append(&mut annotated_excerpts);
            search_result.score = search_result.score.saturating_add(r.result.score);
        }
    }

    let mut output_results: Vec<OutputResult> = combined_excerpts
        .iter()
        .map(|(k, v)| OutputResult::from(&index.entries[*k], v))
        .collect();

    output_results.sort_by(|a, b| b.score.cmp(&a.score));

    SearchOutput {
        results: output_results[0..std::cmp::min(output_results.len(), 10)].to_vec(),
        total_hit_count: output_results.len(),
    }
}
