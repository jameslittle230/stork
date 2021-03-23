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
        // .setting(AppSettings::SubcommandRequiredElseHelp)
        .setting(AppSettings::VersionlessSubcommands)
        .arg(
            Arg::with_name("timing")
                .short("t")
                .long("timing")
                .help("Displays information on the command line about how long an operation took"),
        )
        // .arg(Arg::with_name("quiet").short("q").long("quiet"))
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
                        .takes_value(true)
                        .help("The path of the index file that should be searched.")
                        .required(true),
                )
                .arg(
                    Arg::with_name("query")
                        .short("q")
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
    // .subcommand(SubCommand::with_name("explore-index"))
}
