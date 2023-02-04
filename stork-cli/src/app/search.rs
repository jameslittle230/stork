use clap::{Arg, Command};
use lazy_static::lazy_static;
use stork_lib::SearchConfig;

pub(super) fn search_subcommand() -> Command<'static> {
    lazy_static! {
        static ref DEFAULT_SEARCH_CONFIG: SearchConfig = SearchConfig::default();
        static ref DEFAULT_NUMBER_OF_EXCERPTS: String =
            DEFAULT_SEARCH_CONFIG.number_of_excerpts.to_string();
        static ref DEFAULT_EXCERPT_LENGTH: String =
            DEFAULT_SEARCH_CONFIG.excerpt_length.to_string();
        static ref DEFAULT_NUMBER_OF_RESULTS: String =
            DEFAULT_SEARCH_CONFIG.number_of_results.to_string();
    }

    Command::new("search")
        .about("Search an index for a query.")
        .arg(
            Arg::with_name("index")
                .short('i')
                .long("index")
                .takes_value(true)
                .value_name("INDEX_PATH")
                .help("The path of the index file that should be searched.")
                .required(true),
        )
        .arg(
            Arg::with_name("query")
                .short('q')
                .long("query")
                .takes_value(true)
                .value_name("SEARCH_QUERY")
                .help("The text with which to search the index")
                .required(true),
        )
        .next_help_heading("DISPLAY")
        .arg(
            Arg::with_name("format")
                .long("format")
                .takes_value(true)
                .value_name("FORMAT")
                .possible_values(["json", "pretty", "none"])
                .default_value("pretty")
                .help("The output format for the returned search results"),
        )
        .arg(
            Arg::with_name("number_of_excerpts")
                .long("number-of-excerpts")
                .takes_value(true)
                .value_name("NUMBER_OF_EXCERPTS")
                .default_value(&DEFAULT_NUMBER_OF_EXCERPTS)
                .help("The maximum number of excerpts to return for each result."),
        )
        .arg(
            Arg::with_name("number_of_results")
                .long("number-of-results")
                .takes_value(true)
                .value_name("NUMBER_OF_RESULTS")
                .default_value(&DEFAULT_NUMBER_OF_RESULTS)
                .help("The maximum number of documents to return in the search output."),
        )
        .arg(
            Arg::with_name("excerpt_length")
                .long("excerpt-length")
                .takes_value(true)
                .value_name("EXCERPT_LENGTH")
                .default_value(&DEFAULT_EXCERPT_LENGTH)
                .help("The length, in characters, of each text excerpt returned in the search output."),
        )
        .next_help_heading("DIAGNOSTICS")
        .arg(
            Arg::with_name("timing")
                .short('t')
                .long("timing")
                .help("Displays the duration of the search operation"),
        )
}
