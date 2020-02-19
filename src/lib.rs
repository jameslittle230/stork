pub mod config;
pub mod searcher;
mod index_versions;

use config::*;
use console_error_panic_hook;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

use index_versions::v2 as LatestVersion;
use LatestVersion::builder;
use LatestVersion::structs::Index;

type IndexFromFile = [u8];
type Fields = Option<HashMap<String, String>>;

#[wasm_bindgen]
pub fn search(index: &IndexFromFile, query: String) -> String {
    console_error_panic_hook::set_once();
    serde_json::to_string(&searcher::search(index, &query)).unwrap()
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
