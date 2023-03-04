use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;
use ts_rs::TS;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, SmartDefault, TS)]
#[serde(deny_unknown_fields, default)]
#[ts(export)]
#[allow(non_snake_case)]
pub struct OutputConfig {
    // #[default = 0]
    // pub chunk_size_kb: usize,
    #[default = 3]
    pub minimum_query_length: u8,

    #[default = false]
    pub break_on_first_error: bool,
}
