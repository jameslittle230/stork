use bytes::Bytes;
use once_cell::sync::OnceCell;
use serde::Serialize;
use std::{collections::HashMap, convert::From, fmt::Display, sync::Mutex};
use stork_boundary::{IndexMetadata, Output};
use stork_lib::{index_from_bytes, IndexParseError, ParsedIndex, SearchError};

#[cfg(feature = "v3")]
use stork_lib::V3Search;

#[cfg(feature = "v2")]
use stork_lib::V2Search;

use thiserror::Error;
use wasm_bindgen::prelude::*;

// We can't pass a parsed index over the WASM boundary so we store the parsed indices here
static INDEX_CACHE: OnceCell<Mutex<HashMap<String, ParsedIndex>>> = OnceCell::new();

struct JsonSerializationError {}

impl Display for JsonSerializationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Could not convert Stork data to JSON. If you see this, please file a bug: https://jil.im/storkbug")
    }
}

struct WasmOutput(String);

impl<T: Sized + Serialize, E: Display> From<Result<T, E>> for WasmOutput {
    fn from(r: Result<T, E>) -> Self {
        fn wasm_format_error<E: Display>(e: E) -> String {
            format!("{{\"error\": \"{}\"}}", e)
        }

        let value = match r {
            Ok(object) => match serde_json::to_string(&object) {
                Ok(s) => s,
                Err(_e) => wasm_format_error(JsonSerializationError {}),
            },
            Err(e) => wasm_format_error(e),
        };

        WasmOutput(value)
    }
}

#[wasm_bindgen]
pub fn wasm_register_index(name: &str, data: &[u8]) -> String {
    let data = Bytes::from(Vec::from(data)); // TODO: This seems questionable
    console_error_panic_hook::set_once();

    let result_wrap = || -> Result<IndexMetadata, IndexParseError> {
        let index = index_from_bytes(data)?;
        let mut hashmap = INDEX_CACHE
            .get_or_init(|| Mutex::new(HashMap::new()))
            .lock()
            .unwrap();
        let metadata = index.get_metadata();
        hashmap.insert(name.to_string(), index);
        Ok(metadata)
    };

    WasmOutput::from(result_wrap()).0
}

#[derive(Debug, Error)]
pub enum WasmSearchError {
    #[error("{0}")]
    SearchError(#[from] SearchError),

    #[error("Index `{0}` has not been registered. You need to register the index before performing searches with it.")]
    NamedIndexNotInCache(String),
}

#[wasm_bindgen]
pub fn wasm_search(name: &str, query: &str) -> String {
    console_error_panic_hook::set_once();

    let result_wrap = || -> Result<Output, WasmSearchError> {
        let hashmap = INDEX_CACHE
            .get_or_init(|| Mutex::new(HashMap::new()))
            .lock()
            .unwrap();

        let index = hashmap
            .get(name)
            .ok_or_else(|| WasmSearchError::NamedIndexNotInCache(name.to_string()))?;

        #[allow(unreachable_patterns)]
        #[allow(clippy::match_wildcard_for_single_variants)]
        match index {
            #[cfg(feature = "v3")]
            ParsedIndex::V3(index) => Ok(V3Search(index, query)),

            #[cfg(feature = "v2")]
            ParsedIndex::V2(index) => Ok(V2Search(&index, query)),

            _ => Err(WasmSearchError::from(SearchError::IndexVersionNotSupported)),
        }
    };

    WasmOutput::from(result_wrap()).0
}

#[wasm_bindgen]
pub fn wasm_stork_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[cfg(test)]
mod tests {

    #[derive(Serialize)]
    struct MyData {
        one: u32,
        two: String,
        three: bool,
    }

    use std::{
        fs,
        io::{BufReader, Read},
    };

    use super::*;
    #[test]
    fn serializiable_value_can_be_deserialized() {
        let my_data_val = MyData {
            one: 42,
            two: "This is a string".to_string(),
            three: true,
        };

        let result: Result<MyData, JsonSerializationError> = Ok(my_data_val);

        let computed = WasmOutput::from(result).0;
        let expected = "{\"one\":42,\"two\":\"This is a string\",\"three\":true}".to_string();
        assert_eq!(computed, expected);
    }

    #[test]
    fn error_result_gives_error_json() {
        let my_error = JsonSerializationError {};
        let result: Result<MyData, JsonSerializationError> = Err(my_error);

        let computed = WasmOutput::from(result).0;
        let expected = "{\"error\": \"Could not convert Stork data to JSON. If you see this, please file a bug: https://jil.im/storkbug\"}".to_string();
        assert_eq!(computed, expected);
    }

    #[test]
    fn retrieve_v3_from_cache() {
        let file = fs::File::open("../test-assets/federalist-min-0.7.0.st").unwrap();
        let mut buf_reader = BufReader::new(file);
        let mut index_bytes: Vec<u8> = Vec::new();
        let _bytes_read = buf_reader.read_to_end(&mut index_bytes).unwrap();

        let _str = wasm_register_index("zero-seven-zero", index_bytes.as_slice());
        let str = wasm_register_index("zero-zeven-zero-again", index_bytes.as_slice());
        assert_eq!(str, r#"{"indexVersion":"stork-3"}"#);

        let results = wasm_search("zero-seven-zero", "liberty");
        assert!(results.contains("despotic power and hostile to the principles of liberty. An over-scrupulous jealousy of danger to the"));
        assert_eq!(results.len(), 1254);
    }

    #[test]
    fn cache_miss_errors_as_expected() {
        let file = fs::File::open("../test-assets/federalist-min-0.7.0.st").unwrap();
        let mut buf_reader = BufReader::new(file);
        let mut index_bytes: Vec<u8> = Vec::new();
        let _bytes_read = buf_reader.read_to_end(&mut index_bytes).unwrap();

        let str = wasm_register_index("cache-name-one", index_bytes.as_slice());
        assert_eq!(str, r#"{"indexVersion":"stork-3"}"#);

        let results = wasm_search("cache-name-two", "liberty");
        dbg!(&results);
        assert_eq!(
            results,
            r#"{"error": "Index `cache-name-two` has not been registered. You need to register the index before performing searches with it."}"#
        );
    }

    #[cfg(feature = "v2")]
    #[test]
    fn retrieve_v2_from_cache() {
        let file = fs::File::open("../test-assets/federalist-min-0.5.3.st").unwrap();
        let mut buf_reader = BufReader::new(file);
        let mut index_bytes: Vec<u8> = Vec::new();
        let _bytes_read = buf_reader.read_to_end(&mut index_bytes).unwrap();

        let str = wasm_register_index("zero-five-three", index_bytes.as_slice());
        assert_eq!(str, r#"{"indexVersion":"stork-2"}"#);

        let results = wasm_search("zero-five-three", "liberty");
        assert!(results.contains("despotic power and hostile to the principles of liberty. An over-scrupulous jealousy of danger to the"));
        assert_eq!(results.len(), 1254);
    }
}
