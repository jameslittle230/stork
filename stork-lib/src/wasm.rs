use crate::{parse_and_cache_index, search_from_cache, ParsedIndex};
use bytes::Bytes;
use serde::Serialize;
use std::{convert::From, fmt::Display};
use stork_boundary::IndexMetadata;
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

impl From<&ParsedIndex> for IndexMetadata {
    fn from(index: &ParsedIndex) -> Self {
        let index_version = {
            match index {
                ParsedIndex::V2(_) => "stork-2",
                ParsedIndex::V3(_) => "stork-3",
            }
        };

        IndexMetadata {
            index_version: index_version.to_string(),
        }
    }
}

#[wasm_bindgen]
pub fn wasm_register_index(name: &str, data: &[u8]) -> String {
    let data = Bytes::from(Vec::from(data)); // TODO: This seems questionable
    console_error_panic_hook::set_once();
    WasmOutput::from(parse_and_cache_index(data, name).map(IndexMetadata::from)).0
}

#[wasm_bindgen]
pub fn wasm_search(name: &str, query: &str) -> String {
    console_error_panic_hook::set_once();
    WasmOutput::from(search_from_cache(name, query)).0
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
}
