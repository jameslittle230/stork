use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;
use ts_rs::TS;

#[derive(Serialize, Deserialize, Debug, Clone, SmartDefault, PartialEq, Eq, TS)]
#[ts(export)]
pub struct SRTConfig {
    #[default = true]
    pub timestamp_linking: bool,

    #[default = "&t={}"]
    pub timestamp_template_string: String,
    pub timestamp_format: SRTTimestampFormat,
}

#[derive(Serialize, Deserialize, Debug, Clone, SmartDefault, PartialEq, Eq, TS)]
#[serde(rename_all = "snake_case")]
pub enum SRTTimestampFormat {
    #[default]
    NumberOfSeconds,
    MinutesAndSeconds,
}
