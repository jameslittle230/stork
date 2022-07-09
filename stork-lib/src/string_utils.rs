pub(crate) fn pluralize_with_count(count: usize, singular: &str, plural: &str) -> String {
    format!("{} {}", count, if count == 1 { singular } else { plural })
}

pub(crate) fn remove_surrounding_punctuation(input: &str) -> String {
    let mut chars: Vec<char> = input.chars().collect();

    while chars.first().unwrap_or(&'a').is_ascii_punctuation() {
        chars.remove(0);
    }

    while chars.last().unwrap_or(&'a').is_ascii_punctuation() {
        chars.pop();
    }

    chars.into_iter().collect()
}

pub(crate) fn get_surroundings(
    string: &str,
    offset: usize,
    context: usize,
) -> (Option<&str>, Option<&str>, Option<&str>) {
    // TODO: This is an optimization based on my assumption that splitting the string by chars
    // before splitting it into words will be faster than just splitting it by words infinitely
    // in both directions. Is this true?
    let char_context = context.saturating_mul(24); // TODO - if words are longer than this, it'll be a problem
    let min_char = offset.saturating_sub(char_context);
    let max_char = std::cmp::min(
        offset.saturating_add(char_context),
        string.len().saturating_sub(1),
    );

    let before_words = segment_words(string.get(min_char..offset).unwrap_or_default());
    let after_words = segment_words(string.get(offset..=max_char).unwrap_or_default());

    if context == 0 || (before_words.is_empty() && after_words.is_empty()) {
        return (None, None, None);
    }

    let before_offset = before_words
        .get(before_words.len().saturating_sub(context))
        .map_or(0, |a| a.character_offset)
        .saturating_add(min_char);

    let after_offset = {
        let last_word = after_words
            .get(context.saturating_sub(1))
            .unwrap_or_else(|| after_words.last().unwrap());

        last_word
            .character_offset
            .saturating_add(offset)
            .saturating_add(last_word.word.len())
    };

    (
        string.get(before_offset..offset),
        string.get(offset..after_offset),
        string.get(before_offset..after_offset),
    )
}

#[derive(Debug, Clone)]
pub(crate) struct IndexedWord {
    pub(crate) word: String,
    pub(crate) character_offset: usize,
}

pub(crate) fn segment_words(string: &str) -> Vec<IndexedWord> {
    let split = string.split(|c: char| c.is_ascii_whitespace() || c == '-');

    let mut indexed_words: Vec<IndexedWord> = Vec::new();
    let mut offset_so_far = 0;

    for thing in split {
        if !thing.trim().is_empty() {
            indexed_words.push(IndexedWord {
                word: thing
                    .to_lowercase()
                    .trim_matches(|char: char| char.is_ascii_punctuation())
                    .to_string(),
                character_offset: offset_so_far,
            });
        }
        offset_so_far += thing.chars().count() + 1;
    }

    indexed_words
}

#[cfg(test)]
mod tests {

    mod get_surroundings {
        use pretty_assertions::assert_eq;

        use super::super::get_surroundings;

        #[test]
        fn happy_path() {
            assert_eq!(
                get_surroundings("aaaa Lorem ipsum dolor sit amet", 17, 2),
                (
                    Some("Lorem ipsum "),
                    Some("dolor sit"),
                    Some("Lorem ipsum dolor sit")
                )
            );
        }

        #[test]
        fn mid_word() {
            assert_eq!(
                get_surroundings("aa Lorem ipsum dolor sit amet", 17, 2),
                (Some("ipsum do"), Some("lor sit"), Some("ipsum dolor sit"))
            );
        }

        #[test]
        fn empty_string() {
            assert_eq!(get_surroundings("", 17, 2), (None, None, None));
        }
        #[test]
        fn single_word_string() {
            let value = get_surroundings("howdy", 0, 1);
            assert_eq!(value, (Some(""), Some("howdy"), Some("howdy")));
        }

        fn get_surroundings_works_for_long_strings() {}
    }
}
