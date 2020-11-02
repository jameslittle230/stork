use std::fmt::Display;

use crate::{common::IndexFromFile, searcher::parse::IndexMetadata};
use serde::Serialize;
use wasm_bindgen::prelude::*;

use super::{parse_index, search};

struct JsonSerializationError {}

impl Display for JsonSerializationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Could not convert Stork data to JSON. If you see this, please file a bug: https://jil.im/storkbug")
    }
}

fn wasm_output<T: Sized + Serialize, E: Display>(r: Result<T, E>) -> String {
    fn wasm_format_error<E: Display>(e: E) -> String {
        format!("{{\"error\": \"{}\"}}", e)
    }

    match r {
        Ok(object) => match serde_json::to_string(&object) {
            Ok(s) => s,
            Err(_e) => wasm_format_error(JsonSerializationError {}),
        },
        Err(e) => wasm_format_error(e),
    }
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub fn wasm_register_index(name: &str, data: &IndexFromFile) -> String {
    console_error_panic_hook::set_once();
    wasm_output(parse_index(data, name).map(|index| {
        IndexMetadata::from(index)
    }))
}

#[wasm_bindgen]
pub fn wasm_search(name: &str, query: &str) -> String {
    console_error_panic_hook::set_once();
    wasm_output(search(name, query))
}
