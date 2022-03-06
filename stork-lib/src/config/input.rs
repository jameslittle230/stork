use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use super::{File, FrontmatterConfig, SRTConfig, StemmingConfig};

#[derive(Serialize, Deserialize, Clone, Debug, SmartDefault, PartialEq)]
#[serde(deny_unknown_fields)]
pub enum TitleBoost {
    Minimal,
    #[default]
    Moderate,
    Large,
    Ridiculous,
}

#[derive(Serialize, Deserialize, Debug, Clone, SmartDefault, PartialEq)]
#[serde(deny_unknown_fields, default)]
#[allow(non_snake_case)]
pub struct InputConfig {
    #[serde(rename = "surrounding_word_count")]
    pub UNUSED_surrounding_word_count: Option<u8>,
    pub base_directory: String,
    pub url_prefix: String,
    pub title_boost: TitleBoost,
    pub stemming: StemmingConfig,
    pub html_selector: Option<String>,

    #[default(None)]
    pub exclude_html_selector: Option<String>,
    pub frontmatter_handling: FrontmatterConfig,
    pub files: Vec<File>,

    #[default = false]
    pub break_on_file_error: bool,
    pub srt_config: SRTConfig,

    #[default = 3]
    pub minimum_indexed_substring_length: u8,

    #[default = 1]
    pub minimum_index_ideographic_substring_length: u8,
}
