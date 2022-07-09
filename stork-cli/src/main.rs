#![warn(clippy::pedantic)]

use std::{process::exit, time::Instant};

use colored::Colorize;
use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};

mod clap;
mod display_timings;
mod errors;
mod io;
mod pretty_print_search_results;
mod string_utils;

#[cfg(feature = "test-server")]
mod test_server;

use crate::{clap::app, string_utils::truncate_with_ellipsis_to_length};
use io::{read_bytes_from_path, read_from_path, write_bytes};

use ::clap::ArgMatches;
use errors::StorkCommandLineError;
use num_format::{Locale, ToFormattedString};
use pretty_print_search_results::pretty_print_search_results;
use stork_lib::{build_index, search, Config};

pub type ExitCode = i32;
pub const EXIT_SUCCESS: ExitCode = 0;
pub const EXIT_FAILURE: ExitCode = 1;

type CmdResult = Result<(), StorkCommandLineError>;

#[cfg(not(feature = "build"))]
fn main() {
    eprintln!("This binary requires the `build-index-v3` feature to be enabled.");
    exit(EXIT_FAILURE);
}

#[cfg(feature = "build")]
fn main() {
    let app_matches = app().get_matches();

    let result = match app_matches.subcommand() {
        ("build", Some(submatches)) => build_handler(submatches),
        ("search", Some(submatches)) => search_handler(submatches),
        ("test", Some(submatches)) => test_handler(submatches),
        (_, _) => Ok(()),
    };

    if let Err(error) = result {
        eprintln!("{} {}", "Error:".red(), error);
        exit(EXIT_FAILURE);
    }
}

fn build_handler(submatches: &ArgMatches) -> CmdResult {
    let start_time = Instant::now();

    let config_path = submatches.value_of("config").unwrap();
    let output_path = submatches.value_of("output").unwrap();

    let config_string = read_from_path(config_path)?;
    let config = Config::try_from(config_string.as_str())?;

    let pb = ProgressBar::new(config.input.files.len() as u64);
    pb.set_draw_target(ProgressDrawTarget::stderr_nohz());
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed}] {bar:40.cyan/blue} {pos}/{len} | {msg}")
            .progress_chars("##-"),
    );

    let build_output = build_index(&config, &|report| match report {
        stork_lib::BuildProgressReport::StartingDocument {
            count,
            total,
            title,
        } => {
            let message = truncate_with_ellipsis_to_length(&title, 21, None);
            pb.set_message(message);
            pb.set_position(count);
            pb.tick();
        }
        stork_lib::BuildProgressReport::Finished => pb.finish(),
    })?;

    let build_time = Instant::now();

    let bytes_written = write_bytes(output_path, build_output.index.first().unwrap())?;

    let end_time = Instant::now();

    eprintln!(
        "{} Index built successfully, wrote {} bytes.",
        "Success:".green(),
        bytes_written.to_formatted_string(&Locale::en)
    );

    if submatches.is_present("timing") {
        eprintln!(
            "{}",
            display_timings![
                (build_time.duration_since(start_time), "to build index"),
                (end_time.duration_since(build_time), "to write file"),
                (end_time.duration_since(start_time), "total")
            ],
        );
    }

    Ok(())
}

fn search_handler(submatches: &ArgMatches) -> CmdResult {
    let start_time = Instant::now();

    let path = submatches.value_of("index").unwrap();
    let query = submatches.value_of("query").unwrap();

    let index_bytes = read_bytes_from_path(path)?;

    let read_time = Instant::now();

    let results = search(index_bytes, query)?;

    let end_time = Instant::now();

    match submatches.value_of("format") {
        Some("json") => {
            match serde_json::to_string_pretty(&results).map_err(StorkCommandLineError::from) {
                Ok(json) => println!("{}", json),
                Err(error) => {
                    return Err(error);
                }
            }
        }
        Some("pretty") => {
            println!("{}", pretty_print_search_results(&results));
        }
        _ => {
            let _clap_result = app().print_help();
            return Ok(());
        }
    }
    if submatches.is_present("timing") {
        eprintln!(
            "{}",
            display_timings![
                (read_time.duration_since(start_time), "to read index file"),
                (end_time.duration_since(read_time), "to get search results"),
                (end_time.duration_since(start_time), "total")
            ]
        );
    }

    Ok(())
}

#[cfg(not(feature = "test-server"))]
fn test_handler(_: &ArgMatches) -> CmdResult {
    Err(StorkCommandLineError::NotCompiledWithFeature("Stork was not compiled with test server support. Rebuild the crate with all features to enable the test server.\nIf you don't expect to see this, file a bug: https://jil.im/storkbug"))
}

#[cfg(feature = "test-server")]
fn test_handler(submatches: &ArgMatches) -> CmdResult {
    let port_string = submatches.value_of("port").unwrap();
    let port = port_string
        .parse()
        .map_err(|e| StorkCommandLineError::InvalidPort(port_string.to_string(), e))?;

    if let Some(config_path) = submatches.value_of("config") {
        let config_string = read_from_path(config_path)?;
        let config = Config::try_from(config_string.as_str())?;
        let output = build_index(&config)?;
        test_server::serve(&output.bytes, port).map_err(|_| StorkCommandLineError::ServerError)
    } else if let Some(index_path) = submatches.value_of("index_path") {
        let index = read_bytes_from_path(index_path)?;
        test_server::serve(&index, port).map_err(|_| StorkCommandLineError::ServerError)
    } else {
        unreachable!()
    }
}
