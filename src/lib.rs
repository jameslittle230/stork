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
    let search_result = search(index, query)
        .and_then(|output| serde_json::to_string(&output).map_err(|_e| SearchError {}));

    match search_result {
        Ok(string) => string,
        Err(_e) => "{error: 'Could not perform search.'}".to_string(),
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

pub fn build(config: &Config) -> Index {
    builder::build(config)
}

pub fn write(config: &Config, index: Index) -> usize {
    index.write(config)
}
