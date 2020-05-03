pub mod common;
pub mod config;
pub mod index_analyzer;
pub mod searcher;
pub mod stopwords;

mod index_versions;

use config::*;
use wasm_bindgen::prelude::*;

use common::IndexFromFile;
use index_versions::v3 as LatestVersion;
use LatestVersion::builder;
use LatestVersion::structs::Index;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub fn search(index: &IndexFromFile, query: String) -> String {
    console_error_panic_hook::set_once();
    let search_output = searcher::search(index, &query);
    serde_json::to_string(&search_output).unwrap_or_else(|_| "{}".to_string())
}

#[wasm_bindgen]
pub fn get_index_version(index: &IndexFromFile) -> String {
    index_analyzer::get_index_version(index)
        .unwrap_or_else(|_| "Could not parse version".to_string())
}

pub fn build(config: &Config) -> Index {
    builder::build(config)
}

pub fn write_index(config: &Config, index: Index) -> usize {
    index.write(config)
}
