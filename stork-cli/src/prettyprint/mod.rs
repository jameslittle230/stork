use std::cmp::min;

use colored::Colorize;
use stork_lib::search_output::{HighlightRange, SearchOutput};
use textwrap::termwidth;

#[allow(clippy::ptr_arg, clippy::unnecessary_to_owned)]
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

#[allow(clippy::ptr_arg, clippy::unnecessary_to_owned)]
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

pub fn print(search_output: &SearchOutput) -> String {
    let mut output = String::new();

    output.push_str(&format!(
        "{} results found for \"{}\"",
        search_output.total_hit_count.to_string().bold(),
        search_output.query.yellow()
    ));

    let textwrap_options = textwrap::Options::new(min(120, termwidth()))
        .initial_indent("    - ")
        .subsequent_indent("      ");

    search_output.results.iter().for_each(|result| {
        output.push_str("\n\n");
        output.push_str(&format!(
            "{}\n<{}{}>",
            highlight_title(&result.entry.title, &result.title_highlight_ranges),
            search_output.url_prefix,
            result.entry.url
        ));
        result.excerpts.iter().for_each(|excerpt| {
            output.push_str(&format!(
                "\n{}",
                textwrap::fill(
                    &highlight_string(&excerpt.text, &excerpt.highlight_ranges),
                    &textwrap_options
                )
            ));
        });
    });

    output
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    // use pretty_assertions::assert_eq;

    use stork_lib::search_output;

    #[test]
    #[ignore = "reason"]
    fn display_pretty_search_results_given_output() {
        let results = search_output::SearchOutput {
            query: "search query".to_string(),
            results: vec![search_output::SearchResult {
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
                    url_suffix: Some("#25".to_string()),
                    score: 12,
                }],
                title_highlight_ranges: vec![search_output::HighlightRange {
                    beginning: 0,
                    end: 5,
                }],
            }],
            total_hit_count: 21,
            url_prefix: String::new(),
        };

        let computed = print(&results);

        let expected = format!(
            "{}{}{}{}{}{}{}{}{}",
            "21".bold(),
            " results found for \"",
            "search query".yellow(),
            "\"\n\n",
            "Some ".black().bold().on_yellow(),
            "Document Title".bold().green(),
            "\n<https://example.com>\n    - ",
            "T".yellow(),
            "his is the excerpt of the text".normal()
        );

        assert_eq!(
            computed, expected,
            "\n\nCOMPUTED:\n{:?}\n\n\nEXPECTED:\n{:?}\n\n",
            computed, expected,
        );
    }
}
