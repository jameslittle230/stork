use clap::{Arg, Command};

pub(super) fn build_subcommand() -> Command<'static> {
    Command::new("build")
        .about("Builds an index from a configuration and writes it to a file")
        .disable_version_flag(true)
        .arg(
            Arg::new("config")
                .long("input")
                .short('i')
                .help("The path to your configuration file, or - for stdin")
                .takes_value(true)
                .value_name("CONFIG_PATH")
                .required(true),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .takes_value(true)
                .value_name("OUTPUT_PATH")
                .required(true)
                .help("The path of the index file that will be written, or - for stdout"),
        )
        .next_help_heading("DIAGNOSTICS")
        .arg(
            Arg::new("timing")
                .short('t')
                .long("timing")
                .help("Displays the duration of the build operation"),
        )
}
