pub fn remove_surrounding_punctuation(input: &String) -> String {
    let mut chars: Vec<char> = input.chars().collect();

    while chars.first().unwrap().is_ascii_punctuation() {
        chars.remove(0);
    }

    while chars.last().unwrap().is_ascii_punctuation() {
        chars.pop();
    }

    return chars.into_iter().collect();
}
