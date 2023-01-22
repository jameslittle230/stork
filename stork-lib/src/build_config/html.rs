use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, SmartDefault)]
#[serde(deny_unknown_fields, default)]
pub struct HTMLConfig {
    #[default = false]
    pub save_nearest_id: bool,

    #[serde(default)]
    pub included_selectors: Vec<String>,

    #[serde(default)]
    pub excluded_selectors: Vec<String>,
}
