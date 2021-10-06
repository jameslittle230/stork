use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

#[derive(Serialize, Deserialize, Debug, Clone, SmartDefault, PartialEq)]
pub struct SRTConfig {
    #[default = true]
    pub timestamp_linking: bool,

    #[default = "&t={ts}"]
    pub timestamp_template_string: String,
    pub timestamp_format: SRTTimestampFormat,
}

#[derive(Serialize, Deserialize, Debug, Clone, SmartDefault, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SRTTimestampFormat {
    #[default]
    NumberOfSeconds,
}
