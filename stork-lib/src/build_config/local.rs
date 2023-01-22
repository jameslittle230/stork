use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

#[derive(Serialize, Deserialize, Debug, Clone, SmartDefault, PartialEq, Eq)]
pub struct LocalConfig {
    pub debug_output: bool,
}
