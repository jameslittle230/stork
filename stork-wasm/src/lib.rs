use std::{collections::HashMap, sync::Mutex};

use bytes::Bytes;
use lazy_static::lazy_static;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsError;

use stork_lib::{
    parse_index::ParsedIndex,
    search,
    search_value::{SearchValue, SearchValueCacheKey},
};

lazy_static! {
    #[rustfmt::ignore]
    static ref INDEX_CACHE: Mutex<HashMap<String, ParsedIndex>> = Mutex::new(HashMap::new());
    static ref SEARCH_VALUE_CACHE: Mutex<HashMap<SearchValueCacheKey, Vec<SearchValue>>> = Mutex::new(HashMap::new());
}

#[wasm_bindgen]
pub fn wasm_stork_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[wasm_bindgen]
pub fn load_index(name: &str, data: &[u8]) -> Result<String, JsError> {
    if cfg!(debug_assertions) {
        console_error_panic_hook::set_once();
    }

    let bytes = Bytes::from(Vec::from(data));
    let index = stork_lib::parse_bytes_as_index(bytes).map_err(|_| JsError::new("oh no!"))?;
    let stats = index.stats();
    INDEX_CACHE.lock().unwrap().insert(name.to_string(), index);
    Ok(serde_json::to_string(&stats).unwrap())
}

#[wasm_bindgen]
pub fn append_chunk_to_index(name: &str, chunk_data: &[u8]) -> Result<(), JsError> {
    let mut index_cache = INDEX_CACHE.lock().unwrap();
    let index = index_cache.get_mut(name).unwrap(); // TODO: map_err()

    let bytes = Bytes::from(Vec::from(chunk_data));
    stork_lib::add_sidecar_bytes_to_index(index, bytes).map_err(|_ee| JsError::new("asdf"))
}

#[wasm_bindgen]
pub fn perform_search(name: &str, query: &str) -> Result<String, JsError> {
    if cfg!(debug_assertions) {
        console_error_panic_hook::set_once();
    }

    let mut index_cache = INDEX_CACHE.lock().unwrap();
    let index = index_cache.get_mut(name).unwrap(); // TODO: map_err()
    search(index, query)
        .map(|output| serde_json::to_string(&output).unwrap())
        .map_err(|_e| JsError::new("Error"))
}
