use clap::{crate_version, App, AppSettings, Arg, SubCommand};

pub fn app() -> App<'static, 'static> {
    let config_input_arg: Arg = Arg::with_name("config")
        .long("input")
        .short("i")
        .help("The path to your configuration file")
        .takes_value(true)
        .value_name("PATH")
        .required(true);

    App::new("Stork")
        .version(crate_version!())
        .author("James Little <https://jameslittle.me>")
        .about("https://stork-search.net - Impossibly fast web search, made for static sites.")
        // .setting(AppSettings::SubcommandRequiredElseHelp) // TODO: When 2.0.0 is released, uncomment this
        .setting(AppSettings::VersionlessSubcommands)
        .arg(
            Arg::with_name("timing")
                .short("t")
                .long("timing")
                .help("Displays information on the command line about how long an operation took"),
        )
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
                .arg(config_input_arg.clone())
                .arg(
                    Arg::with_name("output")
                        .short("o")
                        .long("output")
                        .takes_value(true)
                        .required(true)
                        .help("The path of the index file that will be written"),
                )
                .display_order(1),
        )
        .subcommand(
            SubCommand::with_name("search")
                .about("Search an index for a query.")
                .arg(
                    Arg::with_name("index")
                        .short("i")
                        .long("index")
                        .takes_value(true)
                        .help("The path of the index file that should be searched.")
                        .required(true),
                )
                .arg(
                    Arg::with_name("query")
                        .short("q")
                        .long("query")
                        .takes_value(true)
                        .help("The search query to look up")
                        .required(true),
                )
                .arg(
                    Arg::with_name("json")
                        .long("json")
                        .display_order(100)
                        .help("If present, the output will be formatted as JSON."),
                )
                .display_order(2),
        )
        .subcommand(
            SubCommand::with_name("test")
                .about("Serves a test web page so you can experiment with an index you're building")
                .arg(config_input_arg.clone())
                .arg(
                    Arg::with_name("port")
                        .help("The port on which to serve the test web page.")
                        .long("port")
                        .short("p")
                        .default_value("1612")
                        .value_name("PORT")
                        .required(false),
                )
                .display_order(3),
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
            "stork --timing search --index - --query -",
            "stork -t search --index - --query -",
            "stork search --index something.st --query my-query --json",
            "stork search -i something.st -q my-query",
            "stork -t search --index something.st --query my-query --json",
            "stork --timing search -i something.st -q my-query",
            "stork test -p 1620 -i something.st",
            "stork test -i something.st -p 1620",
            "stork test -i something.st",
            "stork --build something.toml",
            "stork --search something.toml my-query",
            "stork --test something.st",
        ];

        for input in valid_inputs {
            app()
                .get_matches_from_safe(input.split(" "))
                .unwrap_or_else(|e| panic!("Error with input {:?}: {}", &input, e));
        }
    }
    #[test]
    fn invalid_command_line_input_fails_to_parse() {
        let invalid_inputs = vec![
            "stork build -i something.toml",
            "stork build --input something.toml",
            "stork search -i something.st -j my-query -j",
            "stork --build something.toml --input asdf",
            "stork --build something.toml --output asdf",
            "stork search --index something.st",
            "stork search --query my-query",
            "stork search --index - --query - --timing",
            "stork search --index - --query - -t",
            "stork search --index - -t --query -",
            "stork search --index - --timing --query -",
        ];

        for input in invalid_inputs {
            assert!(
                app().get_matches_from_safe(input.split(" ")).is_err(),
                "{} seemed to be a valid input",
                input
            )
        }
    }
}
