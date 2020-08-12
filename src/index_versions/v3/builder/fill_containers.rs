use super::super::scores::*;
use super::{
    remove_surrounding_punctuation, AnnotatedWord, Container, Excerpt, IntermediateEntry,
    SearchResult, WordListSource,
};
use rust_stemmers::Stemmer;
use std::collections::HashMap;

pub fn fill_containers(
    intermediate_entries: &[IntermediateEntry],
    stems: &HashMap<String, Vec<String>>,
    containers: &mut HashMap<String, Container>,
) {
    for (entry_index, entry) in intermediate_entries.iter().enumerate() {
        let words_in_title: Vec<AnnotatedWord> = entry
            .title
            .split_whitespace()
            .map(|w| AnnotatedWord {
                word: w.to_string(),
                ..Default::default()
            })
            .collect();

        let words_in_contents: Vec<AnnotatedWord> = entry.contents.word_list.to_owned();

        let word_lists = vec![
            (WordListSource::Title, words_in_title),
            (WordListSource::Contents, words_in_contents),
        ];

        for (source, word_list) in word_lists {
            for (word_index, annotated_word) in word_list.iter().enumerate() {
                let normalized_word =
                    remove_surrounding_punctuation(&annotated_word.word.to_lowercase());
                if normalized_word.is_empty() {
                    break;
                }

                fill_container_results_map(
                    containers,
                    &normalized_word,
                    word_index,
                    entry_index,
                    annotated_word,
                    source,
                );

                // Step 2B: Fill _other containers'_ aliases maps with the
                // prefixes of this word
                fill_other_containers_alias_maps_with_prefixes(containers, &normalized_word);

                // Step 2C: Fill _other containers'_ alias maps with the
                // reverse-stems of this word
                fill_other_containers_alias_maps_with_reverse_stems(
                    entry,
                    stems,
                    containers,
                    &normalized_word,
                );
            }
        }
    }
}

fn fill_container_results_map(
    containers: &mut HashMap<String, Container>,
    normalized_word: &str,
    word_index: usize,
    entry_index: usize,
    annotated_word: &AnnotatedWord,
    source: WordListSource,
) {
    let results_map = &mut containers
        .entry(normalized_word.to_string())
        .or_insert_with(Container::new)
        .results;

    let entry_result: &mut SearchResult = results_map
        .entry(entry_index)
        .or_insert_with(SearchResult::new);

    entry_result.excerpts.push(Excerpt {
        word_index,
        source,
        internal_annotations: annotated_word.internal_annotations.clone(),
        fields: annotated_word.fields.clone(),
    });
}

fn fill_other_containers_alias_maps_with_prefixes(
    containers: &mut HashMap<String, Container>,
    normalized_word: &str,
) {
    let chars: Vec<char> = normalized_word.chars().collect();
    for n in 3..chars.len() {
        let substring: String = chars[0..n].iter().collect();

        let alises_map = &mut containers
            .entry(substring.clone())
            .or_insert_with(Container::new)
            .aliases;

        let _alias_score = alises_map
            .entry(normalized_word.to_string())
            .or_insert(PREFIX_SCORE - (chars.len() - n) as u8);
    }
}

fn fill_other_containers_alias_maps_with_reverse_stems(
    entry: &IntermediateEntry,
    stems: &HashMap<String, Vec<String>>,
    containers: &mut HashMap<String, Container>,
    normalized_word: &str,
) {
    if let Some(stem_algorithm) = entry.stem_algorithm {
        let stem = Stemmer::create(stem_algorithm)
            .stem(&normalized_word)
            .to_string();
        if let Some(reverse_stems_vector) = stems.get(&stem) {
            for reverse_stem in reverse_stems_vector {
                if reverse_stem != normalized_word {
                    let _alias_score = containers
                        .entry(reverse_stem.clone())
                        .or_insert_with(Container::new)
                        .aliases
                        .entry(normalized_word.to_string())
                        .or_insert(STEM_SCORE as u8);
                }
            }
        }
    }
}
