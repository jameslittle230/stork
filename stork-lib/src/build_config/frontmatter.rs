#![allow(clippy::module_name_repetitions)]

use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

#[derive(Serialize, Deserialize, Debug, Clone, SmartDefault, PartialEq, Eq)]
pub enum FrontmatterConfig {
    Ignore,
    #[default]
    Omit,
    Parse,
}
