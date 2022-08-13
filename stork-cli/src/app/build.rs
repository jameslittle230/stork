use clap::{Arg, Command};

pub(super) fn build_subcommand() -> Command<'static> {
    Command::new("build")
        .about("Builds an index from a configuration and writes it to a file")
        .disable_version_flag(true)
        .arg(
            Arg::with_name("config")
                .long("input")
                .short('i')
                .help("The path to your configuration file, or - for stdin")
                .takes_value(true)
                .value_name("CONFIG_PATH")
                .required(true),
        )
        .arg(
            Arg::with_name("output")
                .short('o')
                .long("output")
                .takes_value(true)
                .value_name("OUTPUT_PATH")
                .required(true)
                .help("The path of the index file that will be written, or - for stdout"),
        )
        .next_help_heading("DIAGNOSTICS")
        .arg(
            Arg::with_name("timing")
                .short('t')
                .long("timing")
                .help("Displays the duration of the build operation"),
        )
        .arg(
            Arg::with_name("debug")
                .long("debug")
                .help("The output format for the returned search results"), // TODO: Wire this config option up
        )
}
