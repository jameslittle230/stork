use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum FrontmatterConfig {
    Ignore,
    Omit,
    Parse,
}

impl Default for FrontmatterConfig {
    fn default() -> Self {
        FrontmatterConfig::Omit
    }
}
