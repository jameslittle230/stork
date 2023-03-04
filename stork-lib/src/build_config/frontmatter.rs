#![allow(clippy::module_name_repetitions)]

use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;
use ts_rs::TS;

#[derive(Serialize, Deserialize, Debug, Clone, SmartDefault, PartialEq, Eq, TS)]
#[ts(export)]
pub enum FrontmatterConfig {
    Ignore,
    #[default]
    Omit,
    Parse,
}
