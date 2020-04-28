pub mod config;
pub mod index_analyzer;
mod index_versions;
pub mod searcher;

use config::*;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

use index_versions::v2 as LatestVersion;
use LatestVersion::builder;
use LatestVersion::structs::Index;

type IndexFromFile = [u8];
type Fields = Option<HashMap<String, String>>;

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
    index_analyzer::get_index_version(index).unwrap_or("Could not parse version".to_string())
}

pub fn build(config: &Config) -> Index {
    builder::build(config)
}

pub fn write_index(config: &Config, index: Index) -> usize {
    index.write(config)
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
