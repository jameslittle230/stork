#![warn(clippy::pedantic)]

mod app;
mod errors;
mod io;
mod prettyprint;
mod progress;
#[cfg(feature = "test-server")]
mod test_server;
mod timings;

use clap::ArgMatches;
use colored::Colorize;
use lib::SearchConfig;
use num_format::{Locale, ToFormattedString};
use std::time::Instant;

use stork_lib as lib;

type CommandOutput = Result<(), errors::CommandLineError>;

fn main() {
    let app_matches = app::app().get_matches();

    let result = match app_matches.subcommand() {
        Some(("build", submatches)) => build(submatches),
        Some(("search", submatches)) => search(submatches),
        Some(("test", submatches)) => test(submatches),
        _ => unreachable!(),
    };

    match result {
        Ok(_) => {}
        Err(e) => eprintln!("{} {e}", "Error:".red().bold()),
    }
}

fn build(submatches: &ArgMatches) -> CommandOutput {
    let start_time = Instant::now();

    let config_path = submatches.get_one::<String>("config").unwrap();
    let output_path = submatches.get_one::<String>("output").unwrap();

    let config: lib::build_config::Config = io::read(config_path)?.try_into()?;

    let read_config_time = Instant::now();

    let bar = progress::Bar::new(&config);
    let build_output = lib::build_index(&config, bar)?;

    let build_time = Instant::now();

    let bytes_written = io::write(output_path, &build_output.primary_data)?;

    let end_time = Instant::now();

    eprintln!(
        "{} Index built successfully{}. Wrote {} bytes to disk.",
        "Success:".green(),
        if build_output.warnings.is_empty() {
            String::new()
        } else {
            format!(
                " with {} warning{}",
                build_output.warnings.len(),
                if build_output.warnings.len() == 1 {
                    ""
                } else {
                    "s"
                }
            )
        },
        bytes_written.to_formatted_string(&Locale::en)
    );

    if submatches.is_present("timing") {
        print![
            (
                read_config_time.duration_since(start_time),
                "to read config"
            ),
            (
                build_time.duration_since(read_config_time),
                "to build index"
            ),
            (end_time.duration_since(build_time), "to write file"),
            (end_time.duration_since(start_time), "total")
        ];
    }

    Ok(())
}

fn search(submatches: &ArgMatches) -> CommandOutput {
    let start_time = Instant::now();

    let path = submatches.value_of("index").unwrap();
    let query = submatches.value_of("query").unwrap();

    let index_bytes = io::read(path)?;
    let index = lib::parse_bytes_as_index(index_bytes)?;

    let read_time = Instant::now();

    let search_config = SearchConfig {
        excerpt_length: submatches
            .value_of("excerpt_length")
            .unwrap()
            .parse()
            .unwrap(),
        number_of_results: submatches
            .value_of("number_of_results")
            .unwrap()
            .parse()
            .unwrap(),
        number_of_excerpts: submatches
            .value_of("number_of_excerpts")
            .unwrap()
            .parse()
            .unwrap(),
    };

    let results = lib::search(&index, query, &search_config).unwrap();

    let search_time = Instant::now();

    let stdout = match submatches.value_of("format").unwrap() {
        "json" => serde_json::to_string_pretty(&results)?,
        "pretty" => prettyprint::print(&results),
        "none" => String::new(),
        _ => unreachable!(),
    };

    println!("{stdout}");

    let end_time = Instant::now();

    if submatches.is_present("timing") {
        print![
            (read_time.duration_since(start_time), "to read index file"),
            (
                search_time.duration_since(read_time),
                "to get search results"
            ),
            (end_time.duration_since(search_time), "to print results"),
            (end_time.duration_since(start_time), "total")
        ];
    }

    Ok(())
}

#[cfg(not(feature = "test-server"))]
fn test(_: &ArgMatches) -> CommandOutput {
    Err(errors::CommandLineError::NotCompiledWithFeature("Stork was not compiled with test server support. Rebuild the crate with all features to enable the test server.\nIf you don't expect to see this, file a bug: https://jil.im/storkbug"))
}

#[cfg(feature = "test-server")]
fn test(submatches: &ArgMatches) -> CommandOutput {
    let port_string = submatches.value_of("port").unwrap();
    let port = submatches
        .try_get_one::<u16>("port")
        .map_err(|e| errors::CommandLineError::InvalidPort(port_string.to_string(), e))?
        .unwrap()
        .to_owned();

    if let Some(config_path) = submatches.value_of("config") {
        let config: lib::build_config::Config = io::read(config_path)?.try_into()?;
        let bar = progress::Bar::new(&config);
        let build_output = lib::build_index(&config, bar)?;
        test_server::serve(&build_output.primary_data, port)
            .map_err(|_| errors::CommandLineError::ServerError)
    } else if let Some(index_path) = submatches.value_of("index_path") {
        let index = io::read(index_path)?;
        test_server::serve(&index, port).map_err(|_| errors::CommandLineError::ServerError)
    } else {
        unreachable!()
    }
}
