use std::cmp::min;

use colored::Colorize;
use stork_lib::{HighlightRange, Output};
use textwrap::termwidth;

fn highlight_string(string: &str, ranges: &Vec<HighlightRange>) -> String {
    let mut highlighted = String::new();

    let mut last_end = 0;
    for range in ranges {
        highlighted.push_str(&string[last_end..range.beginning]);
        highlighted.push_str(&string[range.beginning..range.end].yellow().to_string());
        last_end = range.end;
    }
    highlighted.push_str(&string[last_end..]);
    highlighted
}

pub fn pretty_print_search_results(results: &Output) -> String {
    let mut output = String::new();

    let textwrap_options = textwrap::Options::new(min(120, termwidth()))
        .initial_indent("    - ")
        .subsequent_indent("      ");

    for result in results.results.iter() {
        output.push_str(&format!(
            "{}\n<{}{}>",
            result.entry.title.bold().green(),
            results.url_prefix,
            result.entry.url
        ));
        for excerpt in result.excerpts.iter() {
            output.push_str(&format!(
                "\n{}",
                textwrap::fill(
                    &highlight_string(&excerpt.text, &excerpt.highlight_ranges),
                    &textwrap_options
                )
            ));
        }
        output.push_str("\n\n");
    }

    output.push_str(&format!(
        "{} total results available",
        results.total_hit_count
    ));

    output
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn display_pretty_search_results_given_output() {
        let results = Output {
            results: vec![stork_lib::StorkResult {
                entry: stork_lib::Entry {
                    title: "Some Document Title".to_string(),
                    url: "https://example.com".to_string(),
                    fields: HashMap::new(),
                },
                score: 25,
                excerpts: vec![stork_lib::Excerpt {
                    text: "This is the excerpt of the text".to_string(),
                    highlight_ranges: vec![stork_lib::HighlightRange {
                        beginning: 0,
                        end: 1,
                    }],
                    internal_annotations: vec![stork_lib::InternalWordAnnotation::UrlSuffix(
                        "#25".to_string(),
                    )],
                    fields: HashMap::new(),
                    score: 12,
                }],
                title_highlight_ranges: vec![stork_lib::HighlightRange {
                    beginning: 0,
                    end: 5,
                }],
            }],
            total_hit_count: 21,
            url_prefix: "".to_string(),
        };

        assert_eq!(
            pretty_print_search_results(&results),
            format!(
                "{}{}{}{}",
                "Some Document Title".bold().green(),
                "\n<https://example.com>\n    - ",
                "T".yellow(),
                "his is the excerpt of the text\n\n21 total results available".normal()
            )
        );
    }
}
