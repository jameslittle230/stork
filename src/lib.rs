pub mod common;
pub mod config;
pub mod searcher;

mod index_versions;

use common::IndexFromFile;
use config::Config;
pub use index_versions::ParsedIndex;
use index_versions::IndexParseError;
use searcher::index_analyzer::parse_index_version;
use searcher::SearchError;

pub use index_versions::v3 as LatestVersion;
use LatestVersion::builder;
use LatestVersion::builder::IndexGenerationError;
use LatestVersion::structs::Index;

use std::convert::TryFrom;
use std::sync::Mutex;
use once_cell::sync::OnceCell;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

// Because we can't pass a parsed index over the WASM boundary,
// we'll store the parsed indices here and give a ticket back
// to the consumer to look up the parsed index later.
static PARSED_INDICES: OnceCell<Mutex<Vec<ParsedIndex>>> = OnceCell::new();
type IndexTicket = usize;

#[wasm_bindgen]
pub fn wasm_parse_index(index: &IndexFromFile) -> Result<IndexTicket, JsValue> {
    let parsed_indices = PARSED_INDICES.get_or_init(|| Mutex::new(Vec::new()));
    let mut vec = parsed_indices.lock().unwrap();
    let ticket = vec.len();

    let index = ParsedIndex::try_from(index).map_err(|e| Into::<JsValue>::into(e))?;
    vec.push(index);

    Ok(ticket)
}

#[wasm_bindgen]
pub fn wasm_search(index: IndexTicket, query: &str) -> String {
    console_error_panic_hook::set_once();

    let parsed_indices = PARSED_INDICES.get_or_init(|| Mutex::new(Vec::new()));
    let lock = parsed_indices.lock().unwrap();

    // TODO: remove this unwrap
    let index = lock.get(index).unwrap();

    let search_result = search(index, query).and_then(|output| {
        serde_json::to_string(&output).map_err(|_e| SearchError::JSONSerializationError)
    });

    match search_result {
        Ok(string) => string,

        // Returning error JSON that the JS can parse: see searchData.ts
        Err(e) => format!("{{\"error\": \"{}\"}}", e),
    }
}

#[wasm_bindgen]
pub fn get_index_version(index: &IndexFromFile) -> String {
    let parse_result = parse_index_version(index);

    match parse_result {
        Ok(v) => format!("{}", v),
        Err(e) => format!("{}", e),
    }
}

pub fn parse_index(index: &IndexFromFile) -> Result<ParsedIndex, IndexParseError> {
    ParsedIndex::try_from(index)
}

pub fn search(
    index: &ParsedIndex,
    query: &str,
) -> Result<searcher::SearchOutput, searcher::SearchError> {
    searcher::search(index, query)
}

pub fn build(config: &Config) -> Result<Index, IndexGenerationError> {
    builder::build(config)
}


// TODO: Fix these tests somehow
#[cfg(test)]
mod tests {
    /*
    use super::*;

    #[test]
    fn wasm_parse_error_returns_json_string() {
        let computed = wasm_search(&[0, 0, 0, 0, 255, 255, 255, 255], "my query");
        let expected = "{\"error\": \"Version size `4294967295` is too long; this isn\'t a valid index file.\"}";
        assert_eq!(computed, expected)
    }

    #[test]
    #[ignore = "This panics in index_analyzer.rs"]
    fn short_blob_throws_appropriate_error() {
        let computed = wasm_search(&[255, 255, 255, 255], "my query");
        let expected = "{\"error\": \"Version size `4294967295` is too long; this isn\'t a valid index file.\"}";
        assert_eq!(computed, expected)
    } */
}
