pub mod intermediate_excerpt;
use std::collections::BTreeMap;
use std::collections::HashMap;

use intermediate_excerpt::IntermediateExcerpt;

mod entry_and_intermediate_excerpts;
use entry_and_intermediate_excerpts::EntryAndIntermediateExcerpts;

use crate::stopwords;
use crate::Output;
use crate::Result;
use crate::V3Index as Index;

use super::scores::STOPWORD_SCORE;
use super::AliasTarget;
use super::Container;
use super::Entry;
use super::EntryIndex;
use super::Score;
use super::SearchResult;

pub fn search(index: &Index, query: &str) -> Output {
    let normalized_query = query.to_lowercase();
    let words_in_query: Vec<String> = normalized_query
        .split(|c| c == ' ' || c == '-')
        .map(ToString::to_string)
        .collect();

    // Get the containers for each word in the query, and separate them
    // into intermediate excerpts
    let mut intermediate_excerpts: Vec<IntermediateExcerpt> = words_in_query
        .iter()
        .filter_map(|word| index.containers.get_key_value(word))
        .map(|(word, ctr)| ContainerWithQuery::new(ctr.clone(), word))
        .flat_map(|ctr_query| ctr_query.get_intermediate_excerpts(index))
        .collect();

    for mut ie in &mut intermediate_excerpts {
        if stopwords.contains(&ie.query.as_str()) {
            ie.score = STOPWORD_SCORE;
        }
    }

    let mut excerpts_by_index: BTreeMap<EntryIndex, Vec<IntermediateExcerpt>> = BTreeMap::new();
    for ie in intermediate_excerpts {
        excerpts_by_index
            .entry(ie.entry_index)
            .or_insert_with(Vec::new)
            .push(ie)
    }

    let total_len = &excerpts_by_index.len();

    let mut output_results: Vec<Result> = excerpts_by_index
        .iter()
        .map(|(entry_index, ies)| {
            let data = EntryAndIntermediateExcerpts {
                entry: index.entries[*entry_index].clone(),
                config: index.config.clone(),
                intermediate_excerpts: ies.clone(),
            };
            Result::from(data)
        })
        .collect();
    output_results.sort_by_key(|or| or.entry.title.clone());
    output_results.sort_by_key(|or| -(or.score as i64));
    output_results.truncate(index.config.displayed_results_count as usize);

    Output {
        results: output_results,
        total_hit_count: *total_len,
        url_prefix: index.config.url_prefix.clone(),
    }
}

#[derive(Debug)]
struct ContainerWithQuery {
    results: BTreeMap<EntryIndex, SearchResult>,
    aliases: BTreeMap<AliasTarget, Score>,
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
            if result.excerpts.is_empty() {
                output.push(IntermediateExcerpt {
                    query: self.query.to_string(),
                    entry_index: *entry_index,
                    score: result.score,
                    source: super::WordListSource::Contents,
                    word_index: 0,
                    internal_annotations: vec![],
                    fields: HashMap::new(),
                })
            }

            for excerpt in &result.excerpts {
                output.push(IntermediateExcerpt {
                    query: self.query.to_string(),
                    entry_index: *entry_index,
                    score: result.score,
                    source: excerpt.source,
                    word_index: excerpt.word_index,
                    internal_annotations: excerpt.internal_annotations.clone(),
                    fields: excerpt.fields.clone(),
                })
            }
        }

        // Put alias containers' results in output
        for (alias_target, alias_score) in &self.aliases {
            if let Some(target_container) = index.containers.get(alias_target) {
                for (entry_index, result) in target_container.results.clone() {
                    if result.excerpts.is_empty() {
                        output.push(IntermediateExcerpt {
                            query: self.query.to_string(),
                            entry_index,
                            score: result.score,
                            source: super::WordListSource::Contents,
                            word_index: 0,
                            internal_annotations: vec![],
                            fields: HashMap::new(),
                        })
                    }

                    for excerpt in result.excerpts.clone() {
                        output.push(IntermediateExcerpt {
                            query: alias_target.to_string(),
                            entry_index,
                            score: *alias_score,
                            internal_annotations: excerpt.internal_annotations,
                            source: excerpt.source,
                            word_index: excerpt.word_index,
                            fields: excerpt.fields,
                        })
                    }
                }
            }
        }

        output
    }
}

impl From<Entry> for crate::Entry {
    fn from(entry: Entry) -> Self {
        crate::Entry {
            url: entry.url.clone(),
            title: entry.title.clone(),
            fields: entry.fields,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;
    use std::fs;
    use std::io::{BufReader, Read};
    #[test]
    fn e2e_v3_search_works() {
        let file = fs::File::open("../test-assets/federalist-min-0.7.0.st").unwrap();
        let mut buf_reader = BufReader::new(file);
        let mut index_bytes: Vec<u8> = Vec::new();
        let _bytes_read = buf_reader.read_to_end(&mut index_bytes);

        let index = Index::try_from(index_bytes.as_slice()).unwrap();
        let generated = search(&index, "liber old world");
        let expected = serde_json::from_str("{\"results\":[{\"entry\":{\"url\":\"https://www.congress.gov/resources/display/content/The+Federalist+Papers#TheFederalistPapers-1\",\"title\":\"Introduction\",\"fields\":{}},\"excerpts\":[{\"text\":\"in many respects the most interesting in the world. It has been frequently remarked that it\",\"highlight_ranges\":[{\"beginning\":45,\"end\":50}],\"score\":128,\"internal_annotations\":[],\"fields\":{}},{\"text\":\"despotic power and hostile to the principles of liberty. An over-scrupulous jealousy of danger to the\",\"highlight_ranges\":[{\"beginning\":48,\"end\":55}],\"score\":125,\"internal_annotations\":[],\"fields\":{}},{\"text\":\"of love, and that the noble enthusiasm of liberty is apt to be infected with a\",\"highlight_ranges\":[{\"beginning\":42,\"end\":49}],\"score\":125,\"internal_annotations\":[],\"fields\":{}},{\"text\":\"of government is essential to the security of liberty; that, in the contemplation of a sound\",\"highlight_ranges\":[{\"beginning\":46,\"end\":53}],\"score\":125,\"internal_annotations\":[],\"fields\":{}},{\"text\":\"that this is the safest course for your liberty, your dignity, and your happiness. I affect\",\"highlight_ranges\":[{\"beginning\":40,\"end\":47}],\"score\":125,\"internal_annotations\":[],\"fields\":{}}],\"title_highlight_ranges\":[],\"score\":878}],\"total_hit_count\":1,\"url_prefix\":\"\"}").unwrap();

        assert_eq!(generated, expected, "{:?}", generated);
    }
}
