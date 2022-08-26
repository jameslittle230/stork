use std::cmp::min;

use colored::Colorize;
use stork_lib::search_output::{HighlightRange, SearchResult};
use textwrap::termwidth;

fn highlight_title(string: &str, ranges: &Vec<HighlightRange>) -> String {
    let mut highlighted = String::new();

    let mut r = ranges.clone();
    r.sort_by_key(|range| range.beginning);

    let mut last_end = 0;
    for range in r {
        highlighted.push_str(&string[last_end..range.beginning].green().bold().to_string());
        highlighted.push_str(
            &string[range.beginning..range.end]
                .black()
                .bold()
                .on_yellow()
                .to_string(),
        );
        last_end = range.end;
    }
    highlighted.push_str(&string[last_end..].green().bold().to_string());
    highlighted
}

fn highlight_string(string: &str, ranges: &Vec<HighlightRange>) -> String {
    let mut highlighted = String::new();

    let mut r = ranges.clone();
    r.sort_by_key(|range| range.beginning);

    let mut last_end = 0;
    for range in r {
        highlighted.push_str(&string[last_end..range.beginning]);
        highlighted.push_str(&string[range.beginning..range.end].yellow().to_string());
        last_end = range.end;
    }
    highlighted.push_str(&string[last_end..]);
    highlighted
}

pub fn print(results: &SearchResult) -> String {
    let mut output = String::new();

    let textwrap_options = textwrap::Options::new(min(120, termwidth()))
        .initial_indent("    - ")
        .subsequent_indent("      ");

    results.results.iter().for_each(|result| {
        output.push_str(&format!(
            "{}\n<{}{}>",
            highlight_title(&result.entry.title, &result.title_highlight_ranges), // TODO: Figure out how to highlight the sections of titles that should be highlighted
            results.url_prefix,
            result.entry.url
        ));
        result.excerpts.iter().for_each(|excerpt| {
            output.push_str(&format!(
                "\n{}",
                textwrap::fill(
                    &highlight_string(
                        &format!("{} {}", &excerpt.text, excerpt.score),
                        &excerpt.highlight_ranges
                    ),
                    &textwrap_options
                )
            ));
        });
        output.push_str("\n\n");
    });

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

    use stork_lib::search_output;

    #[test]
    fn display_pretty_search_results_given_output() {
        let results = search_output::SearchResult {
            results: vec![search_output::Result {
                entry: search_output::Document {
                    title: "Some Document Title".to_string(),
                    url: "https://example.com".to_string(),
                    fields: HashMap::new(),
                },
                score: 25,
                excerpts: vec![search_output::Excerpt {
                    text: "This is the excerpt of the text".to_string(),
                    highlight_ranges: vec![search_output::HighlightRange {
                        beginning: 0,
                        end: 1,
                    }],
                    internal_annotations: vec![search_output::InternalWordAnnotation::UrlSuffix(
                        "#25".to_string(),
                    )],
                    fields: HashMap::new(),
                    score: 12,
                }],
                title_highlight_ranges: vec![search_output::HighlightRange {
                    beginning: 0,
                    end: 5,
                }],
            }],
            total_hit_count: 21,
            url_prefix: "".to_string(),
        };

        assert_eq!(
            print(&results),
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
