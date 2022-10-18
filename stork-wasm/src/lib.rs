use anyhow::Context;
use bytes::Bytes;
use serde::Serialize;
use serde_json::json;
use std::{collections::HashMap, sync::Mutex};

use lazy_static::lazy_static;
use stork_lib::{
    parse_bytes_as_index,
    parse_index::{IndexStatistics, ParsedIndex},
    search_output::SearchResult,
    search_query::SearchTerm,
    search_value::SearchValue,
};
use wasm_bindgen::prelude::*;

// #[wasm_bindgen]
// extern "C" {
//     // Use `js_namespace` here to bind `console.log(..)` instead of just
//     // `log(..)`
//     #[wasm_bindgen(js_namespace = console)]
//     fn log(s: &str);
// }

// macro_rules! clog {
//     // Note that this is using the `log` function imported above during
//     // `bare_bones`
//     ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
// }

fn output_result_as_wasm<T>(result: Result<T, anyhow::Error>) -> String
where
    T: Serialize,
{
    match result {
        Ok(value) => serde_json::to_string(&serde_json::json!({
            "success": true,
            "value": value
        }))
        .unwrap(),
        Err(err) => serde_json::to_string(&serde_json::json!({
            "error": true,
            "message": err.to_string()
        }))
        .unwrap(),
    }
}

#[derive(PartialEq, Eq, Hash, Serialize)]
struct SearchValueCacheKey {
    index_name: String,
    search_term: SearchTerm,
}

lazy_static! {
    static ref INDEX_CACHE: Mutex<HashMap<String, ParsedIndex>> = Mutex::new(HashMap::new());
    static ref SEARCH_VALUE_CACHE: Mutex<HashMap<SearchValueCacheKey, Vec<SearchValue>>> =
        Mutex::new(HashMap::new());
}

#[wasm_bindgen]
pub fn debug() -> String {
    let index_cache = INDEX_CACHE.lock().unwrap();
    let index_cache_debug = index_cache
        .keys()
        .filter_map(|k| {
            let (key, index) = index_cache.get_key_value(k)?;
            Some((key.clone(), index.stats()))
        })
        .collect::<Vec<(String, IndexStatistics)>>();

    let search_value_cache = SEARCH_VALUE_CACHE.lock().unwrap();
    let search_value_cache_debug = search_value_cache
        .keys()
        .filter_map(|k| {
            let (key, index) = search_value_cache.get_key_value(k)?;
            Some((key, index.len()))
        })
        .collect::<Vec<(&SearchValueCacheKey, usize)>>();

    serde_json::to_string(&json!({ "index_cache": index_cache_debug, "search_value_cache": search_value_cache_debug })).unwrap()
}

#[wasm_bindgen]
pub fn wasm_stork_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[wasm_bindgen]
pub fn load_index(name: &str, data: &[u8]) -> String {
    console_error_panic_hook::set_once();
    output_result_as_wasm(load_index_inner(name, data))
}

fn load_index_inner(name: &str, data: &[u8]) -> anyhow::Result<IndexStatistics> {
    let bytes = Bytes::from(Vec::from(data));
    let index = parse_bytes_as_index(bytes).unwrap();
    let stats = index.stats();
    INDEX_CACHE.lock().unwrap().insert(name.to_string(), index);
    Ok(stats)
}

#[wasm_bindgen]
pub fn perform_search(name: &str, query: &str) -> String {
    console_error_panic_hook::set_once();
    output_result_as_wasm(perform_search_inner(name, query))
}

fn perform_search_inner(name: &str, query: &str) -> anyhow::Result<SearchResult> {
    let cache = INDEX_CACHE.lock().unwrap();
    let index = cache
        .get(name)
        .context(format!("Index {name} not found in cache"))?;

    // 1. parse the query string as an array of search terms
    let terms = stork_lib::parse_search_query_string(query);

    // 2. Look up search values from our search term cache
    let value_lists = terms
        .iter()
        .map(|term| {
            let key = SearchValueCacheKey {
                index_name: name.to_string(),
                search_term: term.clone(),
            };

            let mut search_value_cache = SEARCH_VALUE_CACHE.lock().unwrap();
            let cache_value = search_value_cache.get(&key).map(|v| v.to_owned());

            cache_value.unwrap_or_else(|| {
                let values = stork_lib::get_search_values(index, term).unwrap();

                search_value_cache.insert(
                    SearchValueCacheKey {
                        index_name: name.to_string(),
                        search_term: term.clone(),
                    },
                    values.clone(),
                );

                values
            })
        })
        .collect::<Vec<Vec<SearchValue>>>();

    // 3. If we have any new search terms, run get_search_values and cache those new values
    // 4. Merge all the search values and stringify to JSON
    stork_lib::merge_search_values(index, value_lists.clone())
        .context(format!("Can't merge search values {:?}", value_lists))
}
