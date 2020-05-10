use super::scores::*;
use super::structs::*;
use crate::common::{IndexFromFile, STOPWORDS};
use crate::config::TitleBoost;
use crate::searcher::*;
use std::collections::HashMap;
use std::convert::TryFrom;

pub mod intermediate_excerpt;
use intermediate_excerpt::IntermediateExcerpt;

pub fn search(index: &IndexFromFile, query: &str) -> Result<SearchOutput, SearchError> {
    match Index::try_from(index) {
        Err(_e) => Err(SearchError::IndexParseError),
        Ok(index) => {
            let normalized_query = query.to_lowercase();
            let words_in_query: Vec<String> =
                normalized_query.split(' ').map(|s| s.to_string()).collect();

            // Get the containers for each word in the query, and separate them
            // into intermediate excerpts
            let mut intermediate_excerpts: Vec<IntermediateExcerpt> = words_in_query
                .iter()
                .flat_map(|word| index.containers.get_key_value(word))
                .map(|(word, ctr)| ContainerWithQuery::new(ctr.to_owned(), word))
                .map(|ctr_query| ctr_query.get_intermediate_excerpts(&index))
                .flatten()
                .collect();

            for mut ie in &mut intermediate_excerpts {
                if STOPWORDS.contains(&ie.query.as_str()) {
                    ie.score = STOPWORD_SCORE;
                }
            }

            let mut excerpts_by_index: HashMap<EntryIndex, Vec<IntermediateExcerpt>> =
                HashMap::new();
            for ie in intermediate_excerpts {
                excerpts_by_index
                    .entry(ie.entry_index)
                    .or_insert_with(Vec::new)
                    .push(ie)
            }

            let total_len = &excerpts_by_index.len();

            let mut output_results: Vec<OutputResult> = excerpts_by_index
                .iter()
                .map(|(entry_index, ies)| {
                    let data = EntryAndIntermediateExcerpts {
                        entry: index.entries[*entry_index].to_owned(),
                        config: index.config.clone(),
                        intermediate_excerpts: ies.to_owned(),
                    };
                    OutputResult::from(data)
                })
                .collect();
            output_results.sort_by_key(|or| or.entry.title.clone());
            output_results.sort_by_key(|or| -(or.score as i64));
            output_results.truncate(index.config.displayed_results_count as usize);

            Ok(SearchOutput {
                results: output_results,
                total_hit_count: *total_len,
                url_prefix: index.config.url_prefix,
            })
        }
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
        for (entry_index, result) in self.results.iter() {
            for excerpt in result.excerpts.to_owned() {
                output.push(IntermediateExcerpt {
                    query: self.query.to_string(),
                    entry_index: *entry_index,
                    score: result.score,
                    source: excerpt.source,
                    word_index: excerpt.word_index,
                    internal_annotations: excerpt.internal_annotations,
                    fields: excerpt.fields,
                })
            }
        }

        // Put alias containers' results in output
        for (alias_target, alias_score) in self.aliases.iter() {
            if let Some(target_container) = index.containers.get(alias_target) {
                for (entry_index, result) in target_container.results.to_owned() {
                    for excerpt in result.excerpts.to_owned() {
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

impl From<Entry> for OutputEntry {
    fn from(entry: Entry) -> Self {
        OutputEntry {
            url: entry.url.clone(),
            title: entry.title.clone(),
            fields: entry.fields,
        }
    }
}

struct EntryAndIntermediateExcerpts {
    entry: Entry,
    config: PassthroughConfig,
    intermediate_excerpts: Vec<IntermediateExcerpt>,
}

impl From<EntryAndIntermediateExcerpts> for OutputResult {
    fn from(data: EntryAndIntermediateExcerpts) -> Self {
        let entry = data.entry;
        let excerpt_buffer = data.config.excerpt_buffer as usize;
        let split_contents: Vec<String> = entry
            .contents
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();
        let mut ies: Vec<&IntermediateExcerpt> = data
            .intermediate_excerpts
            .iter()
            .filter(|ie| ie.source == WordListSource::Contents)
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

        let mut excerpts: Vec<crate::searcher::Excerpt> = ies_grouped_by_word_index
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

                let score = ies
                    .iter()
                    .map(|ie| (ie.score as usize))
                    .sum::<usize>()
                    .saturating_sub(score_modifier);

                // Since we're mapping from multiple IntermediateExcerpts to one
                // Excerpt, we have to either combine or filter data. For
                // `fields` and `internal_annotations`, I'm taking the data from
                // the first intermediate excerpt in the vector.
                let fields = {
                    if let Some(first) = ies.first() {
                        first.fields.clone()
                    } else {
                        HashMap::new()
                    }
                };

                let internal_annotations = {
                    if let Some(first) = ies.first() {
                        first.internal_annotations.clone()
                    } else {
                        Vec::default()
                    }
                };

                crate::searcher::Excerpt {
                    text,
                    highlight_ranges,
                    internal_annotations,
                    score,
                    fields,
                }
            })
            .collect();

        excerpts.sort_by_key(|e| -(e.score as i16));
        excerpts.truncate(data.config.excerpts_per_result as usize);

        let split_title: Vec<&str> = entry.title.split_whitespace().collect();
        let title_highlight_ranges: Vec<HighlightRange> = data
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

        let title_boost_modifier = title_highlight_ranges.len()
            * match data.config.title_boost {
                TitleBoost::Minimal => 25,
                TitleBoost::Moderate => 75,
                TitleBoost::Large => 150,
                TitleBoost::Ridiculous => 5000,
            };

        let score = if let Some(first) = excerpts.first() {
            first.score
        } else {
            0
        } + title_boost_modifier;

        OutputResult {
            entry: OutputEntry::from(entry),
            excerpts,
            title_highlight_ranges,
            score,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::{BufReader, Read};
    #[test]
    fn e2e_v3_search_works() {
        let file = fs::File::open("./test-assets/federalist-min-1.0.0.st").unwrap();
        let mut buf_reader = BufReader::new(file);
        let mut index_bytes: Vec<u8> = Vec::new();
        let _bytes_read = buf_reader.read_to_end(&mut index_bytes);

        let generated = search(index_bytes.as_slice(), "liber old world").unwrap();
        let expected = serde_json::from_str("{\"results\":[{\"entry\":{\"url\":\"https://www.congress.gov/resources/display/content/The+Federalist+Papers#TheFederalistPapers-1\",\"title\":\"Introduction\",\"fields\":{}},\"excerpts\":[{\"text\":\"in many respects the most interesting in the world. It has been frequently remarked that it\",\"highlight_ranges\":[{\"beginning\":45,\"end\":50}],\"score\":128,\"internal_annotations\":[],\"fields\":{}},{\"text\":\"despotic power and hostile to the principles of liberty. An over-scrupulous jealousy of danger to the\",\"highlight_ranges\":[{\"beginning\":48,\"end\":55}],\"score\":125,\"internal_annotations\":[],\"fields\":{}},{\"text\":\"of love, and that the noble enthusiasm of liberty is apt to be infected with a\",\"highlight_ranges\":[{\"beginning\":42,\"end\":49}],\"score\":125,\"internal_annotations\":[],\"fields\":{}},{\"text\":\"of government is essential to the security of liberty; that, in the contemplation of a sound\",\"highlight_ranges\":[{\"beginning\":46,\"end\":53}],\"score\":125,\"internal_annotations\":[],\"fields\":{}},{\"text\":\"that this is the safest course for your liberty, your dignity, and your happiness. I affect\",\"highlight_ranges\":[{\"beginning\":40,\"end\":47}],\"score\":125,\"internal_annotations\":[],\"fields\":{}}],\"title_highlight_ranges\":[],\"score\":128}],\"total_hit_count\":1,\"url_prefix\":\"\"}").unwrap();

        assert_eq!(generated, expected, "{:?}", generated);
    }
}
