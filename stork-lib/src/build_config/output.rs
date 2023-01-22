use std::default;

use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, SmartDefault)]
#[serde(deny_unknown_fields, default)]
#[allow(non_snake_case)]
pub struct OutputConfig {
    #[default = 0]
    pub chunk_size_kb: usize,

    #[default = 3]
    pub minimum_query_length: u8,

    #[default = 8]
    pub excerpt_buffer: u8,

    #[default = 5]
    pub excerpts_per_result: u8,

    #[default = 10]
    pub displayed_results_count: u8,

    #[default = false]
    pub break_on_file_error: bool,
}
