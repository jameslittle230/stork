use clap::{Arg, Command};

pub(super) fn test_subcommand() -> Command<'static> {
    Command::new("test")
                .about("Serves a test web page so you can experiment with an index you're building.")
                .long_about("Serves a test web page so you can experiment with an index you're building. Pass in either a configuration file or a fully-built index.")
                .arg_required_else_help(true)
                .arg(
                    Arg::with_name("config")
                        .long("config")
                        .short('c')
                        .help("The path to your configuration file, or - for stdin")
                        .takes_value(true)
                        .value_name("CONFIG_PATH")
                        .required(true)
                        .conflicts_with("index_path"),
                )
                .arg(
                    Arg::with_name("index_path")
                        .long("index")
                        .short('i')
                        .help("The path to your index file")
                        .takes_value(true)
                        .value_name("INDEX_PATH")
                        .required(true)
                        .conflicts_with("config"),
                )
                .arg(
                    Arg::with_name("port")
                        .help("The port on which to serve the test web page.")
                        .long("port")
                        .short('p')
                        .default_value("1612")
                        .value_name("PORT")
                        .required(false),
                )
}
