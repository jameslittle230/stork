use std::collections::HashMap;
use rust_stemmers::Stemmer;
use super::{IntermediateEntry, remove_surrounding_punctuation};

pub fn fill_stems(
    intermediate_entries: &Vec<IntermediateEntry>,
    stems: &mut HashMap<String, Vec<String>>,
) {
    for entry in intermediate_entries {
        let contents = &entry.contents;

        if let Some(stem_algorithm) = entry.stem_algorithm {
            for annotated_word in contents.word_list.iter() {
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