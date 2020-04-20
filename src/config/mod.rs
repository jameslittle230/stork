use serde::{Deserialize, Serialize};
use std::fs;

mod file;
pub use file::DataSource;
pub use file::File;
pub use file::Filetype;

pub mod stemming_config;
pub use stemming_config::StemmingConfig;

mod test;

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields, default)]
pub struct Config {
    pub input: InputConfig,
    pub output: OutputConfig,
}

impl Config {
    pub fn from_file(path: std::path::PathBuf) -> Config {
        let contents = fs::read_to_string(&path).unwrap_or_else(|_e| {
            panic!(
                "Something went wrong reading the file {}",
                &path.to_str().unwrap()
            )
        });

        toml::from_str(&contents).expect("Config file does not contain proper TOML syntax.")
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            input: InputConfig::default(),
            output: OutputConfig::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields, default)]
#[allow(non_snake_case)]
pub struct InputConfig {
    #[serde(rename = "surrounding_word_count")]
    pub UNUSED_surrounding_word_count: Option<u8>,
    pub base_directory: String,
    pub url_prefix: String,
    pub stemming: StemmingConfig,
    pub files: Vec<File>,
    pub srt_config: SRTConfig,
}

impl Default for InputConfig {
    fn default() -> Self {
        InputConfig {
            UNUSED_surrounding_word_count: None,
            base_directory: "".to_string(),
            url_prefix: "".to_string(),
            stemming: StemmingConfig::default(),
            files: vec![],
            srt_config: SRTConfig::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields, default)]
pub struct OutputConfig {
    pub filename: String,
    pub debug: bool,
    pub surrounding_word_count: u8,
}

impl Default for OutputConfig {
    fn default() -> Self {
        OutputConfig {
            filename: "output.st".to_string(),
            debug: false,
            surrounding_word_count: 8,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SRTConfig {
    timestamp_linking: bool,
    timestamp_template_string: String,
    timestamp_format: SRTTimestampFormat,
}

impl Default for SRTConfig {
    fn default() -> Self {
        SRTConfig {
            timestamp_linking: true,
            timestamp_template_string: "{ts}".to_string(),
            timestamp_format: SRTTimestampFormat::NumberOfSeconds,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum SRTTimestampFormat {
    NumberOfSeconds,
}
