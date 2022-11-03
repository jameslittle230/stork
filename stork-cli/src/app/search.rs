use clap::{Arg, Command};

pub(super) fn search_subcommand() -> Command<'static> {
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
        .next_help_heading("DIAGNOSTICS")
        .arg(
            Arg::with_name("timing")
                .short('t')
                .long("timing")
                .help("Displays the duration of the search operation"),
        )
        .arg(
            Arg::with_name("format")
                .long("format")
                .display_order(100)
                .takes_value(true)
                .value_name("FORMAT")
                .possible_values(&["json", "pretty", "none"])
                .default_value("pretty")
                .help("The output format for the returned search results"),
        )
}
