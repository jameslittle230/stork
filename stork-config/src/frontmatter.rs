use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

#[derive(Serialize, Deserialize, Debug, Clone, SmartDefault, PartialEq)]
pub enum FrontmatterConfig {
    Ignore,
    #[default]
    Omit,
    Parse,
}
