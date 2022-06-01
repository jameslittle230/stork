use super::{remove_surrounding_punctuation, NormalizedEntry};
use rust_stemmers::Stemmer;
use std::collections::BTreeMap;

pub fn fill_stems(
    intermediate_entries: &[NormalizedEntry],
    stems: &mut BTreeMap<String, Vec<String>>,
) {
    for entry in intermediate_entries {
        let contents = &entry.annotated_word_list;

        if let Some(stem_algorithm) = entry.stem_algorithm {
            for annotated_word in &contents.word_list {
                let normalized_word =
                    remove_surrounding_punctuation(&annotated_word.word.to_lowercase());
                let stem = Stemmer::create(stem_algorithm)
                    .stem(&normalized_word)
                    .to_string();
                let stem_vector = stems.entry(stem).or_insert_with(Vec::default);
                if !stem_vector.contains(&normalized_word) {
                    stem_vector.push(normalized_word);
                }
            }
        }
    }
}
