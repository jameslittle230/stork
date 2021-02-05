use std::{error::Error, fmt, path::PathBuf};
#[derive(Debug)]
pub enum ConfigReadErr {
    EmptyString,
    UnreadableFile(PathBuf),
    UnparseableInput(toml::de::Error),
}

impl Error for ConfigReadErr {}

impl fmt::Display for ConfigReadErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let desc: String = match self {
            ConfigReadErr::EmptyString => "Received empty configuration string".to_string(),
            ConfigReadErr::UnreadableFile(s) => format!("File {} not found", s.to_string_lossy()),
            ConfigReadErr::UnparseableInput(e) => e.to_string(),
        };

        write!(f, "{}", desc)
    }
}
