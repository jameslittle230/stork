extern crate stork_search as stork;

use std::io::stdout;
use std::time::Instant;
use std::{
    fs::File,
    io::{BufWriter, Write},
};
use std::{io::Read, process::exit};

use colored::Colorize;

mod clap;
mod command_line_error;
mod display_timings;
mod test_server;

use crate::clap::app;
use ::clap::ArgMatches;
use command_line_error::StorkCommandLineError;
use num_format::{Locale, ToFormattedString};

use stork::config::Config;
use stork::LatestVersion::structs::Index;

pub type ExitCode = i32;
pub const EXIT_SUCCESS: ExitCode = 0;
pub const EXIT_FAILURE: ExitCode = 1;

fn main() {
    let app_matches = app().get_matches();

    let result = match app_matches.subcommand() {
        ("build", Some(submatches)) => build_handler(submatches, &app_matches),
        ("search", Some(submatches)) => search_handler(submatches, &app_matches),
        ("test", Some(submatches)) => test_handler(submatches, &app_matches),

        // Delete when releasing 2.0.0
        (_, _) => {
            fn print_nudging_string(errant_command: &str) {
                eprintln!("{} The command line interface has been updated: please use `stork {}` instead of `stork --{}`. See `stork --help` for more.", "Warning:".yellow(), errant_command, errant_command)
            }
            if let Some(input_file) = app_matches.value_of("build") {
                print_nudging_string("build");

                let config = Config::from_file(input_file.into());
                match config
                    .map_err(|_| StorkCommandLineError::IndexReadError)
                    .and_then(|config| {
                        config.output.UNUSED_filename.ok_or_else(||
                            StorkCommandLineError::InvalidCommandLineArguments("You've used the old-style command line interface (`stork --build`) with an index file that is missing an output filename, so the output location of your index is ambiguous.".to_string()),
                        )
                    }) {
                    Ok(output_file) => {
                        let global_matches = app().get_matches_from(vec![
                            "stork",
                            "build",
                            "--input",
                            input_file,
                            "--output",
                            &output_file,
                        ]);
                        let submatches = global_matches.subcommand_matches("build").unwrap();
                        build_handler(submatches, &global_matches)
                    }
                    Err(e) => {
                        Err(e)
                    }
                }
            } else if let Some(values_iter) = app_matches.values_of("search") {
                print_nudging_string("search");
                let values: Vec<&str> = values_iter.collect();
                let global_matches = app().get_matches_from(vec![
                    "stork", "search", "--input", values[0], "--query", values[1],
                ]);
                let submatches = global_matches.subcommand_matches("search").unwrap();
                search_handler(submatches, &global_matches)
            } else if let Some(input_file) = app_matches.value_of("test") {
                print_nudging_string("test");
                let global_matches =
                    app().get_matches_from(vec!["stork", "test", "--input", input_file]);
                let submatches = global_matches.subcommand_matches("search").unwrap();
                test_handler(submatches, &global_matches)
            } else {
                let _result = app().print_help();
                Err(StorkCommandLineError::InvalidCommandLineArguments(
                    "Invalid subcommand, expected one of `build`, `search`, or `test`".to_string(),
                ))
            }
        }
    };

    if let Err(error) = result {
        eprintln!("{} {}", "Error:".red(), error);
        exit(EXIT_FAILURE);
    }
}

fn read_stdin_bytes() -> Option<Vec<u8>> {
    use atty::Stream;
    use std::io;

    let mut stdin_buffer = Vec::<u8>::new();
    if atty::isnt(Stream::Stdin) {
        let _bytes_read = io::stdin().read(&mut stdin_buffer);
        return Some(stdin_buffer);
    }

    None
}

fn read_bytes_from_path(path: &str) -> Result<Vec<u8>, StorkCommandLineError> {
    match (path, read_stdin_bytes()) {
        ("-", Some(stdin)) => Ok(stdin),
        ("-", None) => Err(StorkCommandLineError::InteractiveStdinNotAllowed),
        // handle ("", Some) or ("", None), perhaps
        _ => {
            let pathbuf = std::path::PathBuf::from(path);
            std::fs::read(&pathbuf)
                .map_err(|e| StorkCommandLineError::FileReadError(path.to_string(), e))
        }
    }
}

fn read_stdin() -> Option<String> {
    read_stdin_bytes().and_then(|vec| String::from_utf8(vec).ok())
}

fn read_from_path(path: &str) -> Result<String, StorkCommandLineError> {
    match (path, read_stdin()) {
        ("-", Some(stdin)) => Ok(stdin),
        ("-", None) => Err(StorkCommandLineError::InteractiveStdinNotAllowed),
        // handle ("", Some) or ("", None), perhaps
        _ => {
            let pathbuf = std::path::PathBuf::from(path);
            std::fs::read_to_string(&pathbuf)
                .map_err(|e| StorkCommandLineError::FileReadError(path.to_string(), e))
        }
    }
}

#[cfg(not(feature = "build"))]
pub fn build_index(_config: Option<&String>) -> (Config, Index) {
    println!("Stork was not compiled with support for building indexes. Rebuild the crate with default features to enable the test server.\nIf you don't expect to see this, file a bug: https://jil.im/storkbug\n");
    panic!()
}

#[cfg(feature = "build")]
fn build_index(path: &str) -> Result<Index, StorkCommandLineError> {
    let string = read_from_path(path)?;
    let config = Config::from_string(&string).map_err(StorkCommandLineError::ConfigReadError)?;
    stork::build(&config).map_err(StorkCommandLineError::IndexGenerationError)
}

fn write_index_bytes(path: &str, bytes: &[u8]) -> Result<usize, StorkCommandLineError> {
    let mut writer: Box<dyn Write> = if path == "-" {
        Box::new(stdout())
    } else {
        let file = File::create(path)
            .map_err(|e| StorkCommandLineError::FileCreateError(path.to_string(), e))?;
        Box::new(BufWriter::new(file))
    };

    writer
        .write(bytes)
        .map_err(StorkCommandLineError::WriteError)
}

fn build_handler(
    submatches: &ArgMatches,
    global_matches: &ArgMatches,
) -> Result<(), StorkCommandLineError> {
    let start_time = Instant::now();

    let index = build_index(submatches.value_of("config").unwrap())?;
    let output_path = submatches.value_of("output").unwrap();
    let build_time = Instant::now();

    let index_bytes = index.to_bytes();
    let bytes_written = write_index_bytes(output_path, &index_bytes)?;

    let end_time = Instant::now();

    eprintln!(
        "Index built, {} bytes written to {}.",
        bytes_written.to_formatted_string(&Locale::en),
        output_path,
    );

    if global_matches.is_present("timing") {
        eprintln!(
            "{}",
            display_timings![
                (build_time.duration_since(start_time), "to build index"),
                (end_time.duration_since(build_time), "to write file"),
                (end_time.duration_since(start_time), "total")
            ],
        )
    }

    Ok(())
}

fn search_handler(
    submatches: &ArgMatches,
    global_matches: &ArgMatches,
) -> Result<(), StorkCommandLineError> {
    let start_time = Instant::now();

    let path = submatches.value_of("index").unwrap();
    let query = submatches.value_of("query").unwrap();

    let index_bytes = read_bytes_from_path(path)?;
    let read_time = Instant::now();

    let _index_info = stork::parse_and_cache_index(&index_bytes, "a")
        .map_err(|_| StorkCommandLineError::IndexReadError)?;

    let results =
        stork::search_from_cache("a", query).map_err(|_| StorkCommandLineError::SearchError)?;
    let end_time = Instant::now();

    println!("{}", serde_json::to_string_pretty(&results).unwrap());

    if global_matches.is_present("timing") {
        eprintln!(
            "{}",
            display_timings![
                (read_time.duration_since(start_time), "to read index file"),
                (end_time.duration_since(read_time), "to get search results"),
                (end_time.duration_since(start_time), "total")
            ]
        )
    }

    Ok(())
}

fn test_handler(
    submatches: &ArgMatches,
    _global_matches: &ArgMatches,
) -> Result<(), StorkCommandLineError> {
    let port_string = submatches.value_of("port").unwrap();
    let port = port_string
        .parse()
        .map_err(|e| StorkCommandLineError::InvalidPort(port_string.to_string(), e))?;
    let index = build_index(submatches.value_of("config").unwrap())?;
    test_server::serve(&index, port).map_err(|_| StorkCommandLineError::ServerError)
}
