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
    for n in 1..chars.len() {
        let substring: String = chars[0..n].iter().collect();

        if n < 3 && !string_is_cjk_ideographic(&substring) {
            continue;
        }

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

fn string_is_cjk_ideographic(s: &str) -> bool {
    s.chars()
        .map(char_is_cjk_ideograph)
        .fold(true, |acc, x| acc & x)
}


fn char_is_cjk_ideograph(c: char) -> bool {
    // Block ranges sourced from https://en.wikipedia.org/wiki/CJK_Unified_Ideographs#CJK_Unified_Ideographs_blocks
    match c {
        // CJK Unified Ideographs
        '\u{4E00}'..='\u{62FF}' |
        '\u{6300}'..='\u{77FF}' |
        '\u{7800}'..='\u{8CFF}' |
        '\u{8D00}'..='\u{9FFF}' |
        // CJK Unified Ideographs Extension A
        '\u{3400}'..='\u{4DBF}' |
        // CJK Unified Ideographs Extension B
        '\u{20000}'..='\u{215FF}' |
        '\u{21600}'..='\u{230FF}' |
        '\u{23100}'..='\u{245FF}' |
        '\u{24600}'..='\u{260FF}' |
        '\u{26100}'..='\u{275FF}' |
        '\u{27600}'..='\u{290FF}' |
        '\u{29100}'..='\u{2A6DF}' |
        // CJK Unified Ideographs Extension C
        '\u{2A700}'..='\u{2B73F}' |
        // CJK Unified Ideographs Extension D
        '\u{2B740}'..='\u{2B81F}' |
        // CJK Unified Ideographs Extension E
        '\u{2B820}'..='\u{2CEAF}' |
        // CJK Unified Ideographs Extension F
        '\u{2CEB0}'..='\u{2EBEF}' |
        // CJK Unified Ideographs Extension G
        '\u{30000}'..='\u{3134F}' |
        // CJK Compatibility Ideographs
        '\u{F900}'..='\u{FAFF}'
        => true,
        _ => false,
    }
}
