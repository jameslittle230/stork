pub mod common;
pub mod config;
pub mod searcher;

mod index_versions;

use common::IndexFromFile;
use config::Config;
use searcher::index_analyzer::parse_index_version;
use searcher::SearchError;

use index_versions::v3 as LatestVersion;
use LatestVersion::builder;
use LatestVersion::builder::IndexGenerationError;
use LatestVersion::structs::Index;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub fn wasm_search(index: &IndexFromFile, query: String) -> String {
    console_error_panic_hook::set_once();
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

pub fn search(
    index: &IndexFromFile,
    query: String,
) -> Result<searcher::SearchOutput, searcher::SearchError> {
    searcher::search(index, query.as_str())
}

pub fn build(config: &Config) -> Result<Index, IndexGenerationError> {
    builder::build(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn wasm_parse_error_returns_json_string() {
        let computed = wasm_search(&[0, 0, 0, 0, 255, 255, 255, 255], "my query".to_string());
        let expected = "{\"error\": \"Version size `4294967295` is too long; this isn\'t a valid index file.\"}";
        assert_eq!(computed, expected)
    }

    #[test]
    #[ignore = "This panics in index_analyzer.rs"]
    fn short_blob_throws_appropriate_error() {
        let computed = wasm_search(&[255, 255, 255, 255], "my query".to_string());
        let expected = "{\"error\": \"Version size `4294967295` is too long; this isn\'t a valid index file.\"}";
        assert_eq!(computed, expected)
    }
}
