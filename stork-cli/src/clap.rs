use clap::{crate_version, App, AppSettings, Arg, SubCommand};

pub fn app() -> App<'static, 'static> {
    App::new("Stork")
        .bin_name("stork")
        .version(crate_version!())
        .author("James Little <https://jameslittle.me>")
        .about("Impossibly fast web search, made for static sites - https://stork-search.net")
        .setting(AppSettings::VersionlessSubcommands)
        .setting(AppSettings::UnifiedHelpMessage)
        .setting(AppSettings::ColoredHelp)
        .setting(AppSettings::DeriveDisplayOrder)
        .setting(AppSettings::GlobalVersion)
        // .setting(AppSettings::SubcommandRequiredElseHelp) // TODO: When 2.0.0 is released, uncomment this
        .max_term_width(100)
        .arg(
            Arg::with_name("build")
                .takes_value(true)
                .long("build")
                .hidden(true),
        )
        .arg(
            Arg::with_name("test")
                .takes_value(true)
                .long("test")
                .hidden(true),
        )
        .arg(
            Arg::with_name("search")
                .takes_value(true)
                .long("search")
                .min_values(2)
                .hidden(true),
        )
        .subcommand(
            SubCommand::with_name("build")
                .about("Builds an index from a configuration and writes it to a file")
                .arg(Arg::with_name("config")
                    .long("input")
                    .short("i")
                    .help("The path to your configuration file, or - for stdin")
                    .takes_value(true)
                    .value_name("CONFIG_PATH")
                    .required(true))
                .arg(
                    Arg::with_name("output")
                        .short("o")
                        .long("output")
                        .takes_value(true)
                        .value_name("OUTPUT_PATH")
                        .required(true)
                        .help("The path of the index file that will be written, or - for stdout"),
                )
                .arg(
                    Arg::with_name("timing")
                        .short("t")
                        .long("timing")
                        .help("Displays the duration of the build operation"),
                )
        )
        .subcommand(
            SubCommand::with_name("search")
                .about("Search an index for a query.")
                .arg(
                    Arg::with_name("index")
                        .short("i")
                        .long("index")
                        .takes_value(true)
                        .value_name("INDEX_PATH")
                        .help("The path of the index file that should be searched.")
                        .required(true),
                )
                .arg(
                    Arg::with_name("query")
                        .short("q")
                        .long("query")
                        .takes_value(true)
                        .value_name("SEARCH_QUERY")
                        .help("The text with which to search the index")
                        .required(true),
                )
                .arg(
                    Arg::with_name("timing")
                        .short("t")
                        .long("timing")
                        .help("Displays the duration of the search operation"),
                )
                .arg(Arg::with_name("deprecated_json")
                    .long("json")
                    .hidden(true)
                )
                .arg(
                    Arg::with_name("format")
                        .long("format")
                        .display_order(100)
                        .takes_value(true)
                        .value_name("FORMAT")
                        .possible_values(&["json", "pretty"])
                        .default_value("json")
                        .help("The output format for the returned search results"),
                )
        )
        .subcommand(
            SubCommand::with_name("test")
                .about("Serves a test web page so you can experiment with an index you're building.")
                .long_about("Serves a test web page so you can experiment with an index you're building. Pass in either a configuration file or a fully-built index.")
                .arg(
                    Arg::with_name("config")
                        .long("config")
                        .short("c")
                        .help("The path to your configuration file, or - for stdin")
                        .takes_value(true)
                        .value_name("CONFIG_PATH")
                        .required(true)
                        .conflicts_with("index_path"),
                )
                .arg(
                    Arg::with_name("port")
                        .help("The port on which to serve the test web page.")
                        .long("port")
                        .short("p")
                        .default_value("1612")
                        .value_name("PORT")
                        .required(false),
                )
                .arg(
                    Arg::with_name("index_path")
                        .long("index")
                        .short("x")
                        .help("The path to your index file")
                        .takes_value(true)
                        .value_name("INDEX_PATH")
                        .required(true)
                        .conflicts_with("config"),
                )
        )
}

#[cfg(test)]
mod tests {
    use super::app;

    #[test]
    fn valid_command_line_input_parses() {
        let valid_inputs = vec![
            "stork build -i something.toml -o something.st",
            "stork build --input something.toml --output something.st",
            "stork search --index something.st --query my-query",
            "stork search --index - --query -",
            "stork search --index - --query - --timing",
            "stork search --index - --query - -t",
            "stork search --index something.st --query my-query --json",
            "stork search --index something.st --query my-query --format json",
            "stork search --index something.st --query my-query --format pretty",
            "stork search -i something.st -q my-query",
            "stork search -t --index something.st --query my-query --json",
            "stork search --timing -i something.st -q my-query",
            "stork search -t -i something.st -q my-query",
            "stork search --timing -i something.st -q my-query",
            "stork test -p 1620 -c something.toml",
            "stork test -p 1620 -x something.st",
            "stork test -c something.toml -p 1620",
            "stork test -c something.toml",
            "stork test --config something.toml",
            "stork test -x something.st",
            "stork test --index something.st",
            "stork --build something.toml",
            "stork --search something.toml my-query",
            "stork --test something.st",
        ];

        for input in valid_inputs {
            app()
                .get_matches_from_safe(input.split(' '))
                .unwrap_or_else(|e| panic!("Error with input {:?}: {}", &input, e));
        }
    }
    #[test]
    fn invalid_command_line_input_fails_to_parse() {
        let invalid_inputs = vec![
            "stork build -i something.toml",
            "stork build --input something.toml",
            "stork build --o -",
            "stork search -i something.st -j my-query -j",
            "stork --build something.toml --input asdf",
            "stork --build something.toml --output asdf",
            "stork --timing search --index - --query -",
            "stork -t search --index - --query -",
            "stork search --index something.st --query my-query --format bleh",
            "stork search --index something.st",
            "stork search --query my-query",
            "stork test --index something.st --input something.toml",
            "stork test -x something.st -i something.toml",
        ];

        for input in invalid_inputs {
            assert!(
                app().get_matches_from_safe(input.split(' ')).is_err(),
                "{} seemed to be a valid input",
                input
            )
        }
    }
}
