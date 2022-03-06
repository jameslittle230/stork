use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, SmartDefault)]
#[serde(deny_unknown_fields, default)]
#[allow(non_snake_case)]
pub struct OutputConfig {
    #[serde(rename = "filename")]
    #[default(None)]
    pub UNUSED_filename: Option<String>,

    #[default = false]
    pub debug: bool,

    #[default = false]
    pub save_nearest_html_id: bool,

    #[default = 8]
    pub excerpt_buffer: u8,

    #[default = 5]
    pub excerpts_per_result: u8,

    #[default = 10]
    pub displayed_results_count: u8,
}
