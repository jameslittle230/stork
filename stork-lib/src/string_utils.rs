#![allow(dead_code)]

use unicode_segmentation::UnicodeSegmentation;

pub(crate) fn pluralize_with_count(count: usize, singular: &str, plural: &str) -> String {
    format!("{} {}", count, if count == 1 { singular } else { plural })
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct IndexedWord {
    pub(crate) word: String,
    pub(crate) byte_offset: usize,
}

pub(crate) fn split_into_normalized_words(string: &str) -> Vec<IndexedWord> {
    string
        .unicode_word_indices() // TODO: Figure out if and how I should split on more characters, like apostrophes
        .filter_map(|(index, word)| match word.trim().is_empty() {
            false => Some(IndexedWord {
                word: word
                    .to_lowercase()
                    .trim_matches(|char: char| char.is_ascii_punctuation())
                    .to_string(),
                byte_offset: index,
            }),
            true => None,
        })
        .collect()
}

/// Given a string, a byte offset, and a number (n) of surrounding words,
/// this function will return two strings: the first is n words before the byte offset,
/// and the second is n words after the byte offset.
///
/// It is undefined behavior if byte_offset isn't the start of a word as defined
/// by unicode_words().
pub(crate) fn get_words_surrounding_offset(
    string: &str,
    byte_offset: usize,
    surrounding_word_count: usize,
) -> (String, String) {
    let before = &string[0..byte_offset];
    let after = &string[byte_offset..];

    // TODO: Write tests for, and panic on, the undefined behavior described above

    let byte_offset_at_surround_beginning = before
        .unicode_word_indices()
        .nth_back(surrounding_word_count - 1)
        .map(|(index, _string)| index)
        .unwrap_or(0);

    let byte_offset_at_surround_end = after
        .unicode_word_indices()
        .nth(surrounding_word_count - 1)
        // This string.len() at the end lets us calculate the byte offset of the
        // end of the last surrounding word without the word separator between
        // the last surrounding word and the subsequent word.
        .map(|(index, string)| index + byte_offset + string.len())
        .unwrap_or(string.len());

    (
        string[byte_offset_at_surround_beginning..byte_offset].to_string(),
        string[byte_offset..byte_offset_at_surround_end].to_string(),
    )
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::{get_words_surrounding_offset, split_into_normalized_words, IndexedWord};

    #[test]
    fn it_only_gets_n_surrounding_words() {
        assert_eq!(
            get_words_surrounding_offset("a b c d e f g h", 6, 2),
            ("b c ".to_string(), "d e".to_string())
        )
    }

    #[test]
    fn it_splits_strings_with_weird_punctuation() {
        assert_eq!(
            get_words_surrounding_offset("a b---c----d---e-----f g h", 11, 2),
            ("b---c----".to_string(), "d---e".to_string())
        )
    }

    #[test]
    #[ignore = "It doesn't work - see above todo"]
    fn it_splits_apostraphes_into_two_words() {
        assert_eq!(
            split_into_normalized_words("o'neill"),
            vec![
                IndexedWord {
                    word: "o".to_string(),
                    byte_offset: 0
                },
                IndexedWord {
                    word: "neill".to_string(),
                    byte_offset: 2
                }
            ]
        )
    }

    #[test]
    #[ignore = "It doesn't work - see above todo"]
    fn it_splits_apostraphes_into_two_words_with_special_apostrophe() {
        assert_eq!(
            split_into_normalized_words("o’neill"),
            vec![
                IndexedWord {
                    word: "o".to_string(),
                    byte_offset: 0
                },
                IndexedWord {
                    word: "neill".to_string(),
                    byte_offset: 2
                }
            ]
        )
    }

    #[test]
    fn it_splits_normalized_words_with_the_same_offsets() {
        let test_subjects = vec![
            (
                "a b---c----d---e-----f g h",
                3,
                ("b---c----".to_string(), "d---e".to_string()),
            ),
            (
                "if you’re already comfortable with second derivatives, third",
                5,
                (
                    "comfortable with ".to_string(),
                    "second derivatives".to_string(),
                ),
            ),
            (
                "And, if you’re already comfortable with",
                2,
                ("And, if ".to_string(), "you’re already".to_string()),
            ),
            (
                "Higher order derivatives. And, if you’re already comfortable",
                3,
                ("order derivatives. ".to_string(), "And, if".to_string()),
            ),
        ];

        for (string, word_index, expected_surroundings) in test_subjects {
            let split = split_into_normalized_words(string);
            let byte_offset = split.get(word_index).unwrap().byte_offset;
            let surroundings = get_words_surrounding_offset(string, byte_offset, 2);
            assert_eq!(surroundings, expected_surroundings)
        }
    }
}
