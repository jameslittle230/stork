use unicode_segmentation::UnicodeSegmentation;

pub fn truncate_with_ellipsis_to_length(
    string: &str,
    length: usize,
    ellipsis_override: Option<&str>,
) -> String {
    let ellipsis = ellipsis_override.unwrap_or("...");

    let grapheme_iter = UnicodeSegmentation::graphemes(string, true);
    let short_message: String = grapheme_iter.clone().take(length).collect();
    let long_message: String = grapheme_iter.clone().take(length + 1).collect();

    let truncated = {
        let ellipsis = if short_message == long_message {
            ""
        } else {
            ellipsis
        };

        format!("{}{}", short_message, ellipsis)
    };

    truncated
}
