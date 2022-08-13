mod build;
mod search;
mod test;

use clap::{crate_version, AppSettings, Command};

pub fn app() -> Command<'static> {
    Command::new("Stork")
        .bin_name("stork")
        .version(crate_version!())
        .author("James Little <https://jameslittle.me>")
        .about("Impossibly fast web search, made for static sites - https://stork-search.net")
        .propagate_version(true)
        .infer_subcommands(true)
        .subcommand_required(true)
        .arg_required_else_help(true)
        .global_setting(AppSettings::DeriveDisplayOrder)
        .subcommand(build::build_subcommand())
        .subcommand(search::search_subcommand())
        .subcommand(test::test_subcommand())
}

#[cfg(test)]
mod tests {
    use super::app;

    #[test]
    fn verify_app() {
        app().debug_assert();
    }

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
            );
        }
    }
}
