#![allow(unused_imports)]
// For some reason, rustc thinks that `use toml::de::Error as Error;` is an
// unused import, where it's definitely not -- the file will fail to compile
// if I don't import the type.

#[cfg(test)]
use super::Config;
use toml::de::Error;

#[test]
fn empty_file() -> Result<(), Error> {
    let contents = r#""#;
    toml::from_str(contents).map(|_c: Config| ())
}

#[test]
fn simple_config() -> Result<(), Error> {
    let contents = r#"
[input]
base_directory = "test/federalist"
files = [
    {path = "federalist-1.txt", url = "https://www.congress.gov/resources/display/content/The+Federalist+Papers#TheFederalistPapers-1", title = "Introduction"},
    {path = "federalist-2.txt", url = "https://www.congress.gov/resources/display/content/The+Federalist+Papers#TheFederalistPapers-2", title = "Concerning Dangers from Foreign Force and Influence"},
    {path = "federalist-3.txt", url = "https://www.congress.gov/resources/display/content/The+Federalist+Papers#TheFederalistPapers-3", title = "Concerning Dangers from Foreign Force and Influence 2"},
]

[output]
filename = "test/federalist.st"
debug = false
    "#;
    toml::from_str(contents).map(|_c: Config| ())
}

#[test]
fn surrounding_word_count_in_input() -> Result<(), Error> {
    let contents = r#"
[input]
base_directory = "test/federalist"
surrounding_word_count = 2
files = []

[output]
    "#;
    toml::from_str(contents).map(|_c: Config| ())
}

#[test]
fn unknown_key_fails() {
    let contents = r#"
[bad_key]
    "#;
    let result: Result<Config, Error> = toml::from_str(contents);

    match result {
        Result::Ok(_r) => panic!("Config creation didn't fail with an unknown key"),
        Result::Err(_e) => (),
    }
}

#[test]
fn empty_file_not_allowed() {
    let contents = r#"
[input]
files = [{}]
    "#;
    let result: Result<Config, Error> = toml::from_str(contents);

    match result {
        Result::Ok(_r) => panic!("Config creation didn't fail with an empty file object"),
        Result::Err(_e) => (),
    }
}

#[test]
fn file_with_only_title_not_allowed() {
    let contents = r#"
[input]
files = [{title = "Derp"}]
    "#;
    let result: Result<Config, Error> = toml::from_str(contents);

    match result {
        Result::Ok(_r) => panic!("Config creation didn't fail with a file object that only had a title. File objects should have a title, url, and data source."),
        Result::Err(_e) => ()
    }
}

#[test]
fn file_with_title_and_url_not_allowed() {
    let contents = r#"
[[input.files]]
title = "Derp"
url = "blorp"
    "#;
    let result: Result<Config, Error> = toml::from_str(contents);

    match result {
        Result::Ok(_r) => panic!("Config creation didn't fail with a file object that only had a title. File objects should have a title, url, and data source."),
        Result::Err(_e) => ()
    }
}

#[test]
fn file_with_title_url_and_datasource_is_allowed() -> Result<(), Error> {
    let contents = r#"
[[input.files]]
title = "Derp"
url = "blorp"
contents = "According to all known laws of aviation, there is no way that a bee should be able to fly."
    "#;
    toml::from_str(contents).map(|_c: Config| ())
}
