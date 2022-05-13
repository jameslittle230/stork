#![warn(clippy::pedantic)]
#![allow(clippy::must_use_candidate)]

use bytes::Bytes;
use serde::Serialize;
use std::{convert::From, fmt::Display};
use wasm_bindgen::prelude::*;

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
    console_error_panic_hook::set_once();
    let data = Bytes::from(Vec::from(data)); // TODO: This seems questionable
    let result = stork_lib::register_index(name, data);
    WasmOutput::from(result).0
}

#[wasm_bindgen]
pub fn wasm_search(name: &str, query: &str) -> String {
    console_error_panic_hook::set_once();
    let result = stork_lib::search_from_cache(name, query);
    WasmOutput::from(result).0
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
