use std::str::FromStr;

use rust_stemmers::Algorithm;
use serde::{Deserialize, Serialize};

use strum_macros::{Display, EnumString};
use toml::Value;
use ts_rs::TS;

#[derive(Serialize, Debug, Clone, PartialEq, Eq, TS, EnumString, Display)]
#[strum(ascii_case_insensitive)]
#[ts(export)]
pub enum StemmingConfig {
    None,
    Arabic,
    Danish,
    Dutch,
    English,
    Finnish,
    French,
    German,
    Greek,
    Hungarian,
    Italian,
    Norwegian,
    Portuguese,
    Romanian,
    Russian,
    Spanish,
    Swedish,
    Tamil,
    Turkish,
}

impl Default for StemmingConfig {
    fn default() -> Self {
        StemmingConfig::English
    }
}

impl<'de> serde::Deserialize<'de> for StemmingConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;

        if let Ok(Value::String(string)) = Deserialize::deserialize(deserializer) {
            StemmingConfig::from_str(&string).map_err(|_e| {
                serde::de::Error::custom(format!("Unexpected value `{}`, expected `none` or a language supported by https://snowballstem.org/, e.g. `Dutch`", string.clone()))
            })
        } else {
            Err(Error::custom(
            "Unexpected stemming config value; could not parse as string. (Maybe you need quotes?)",
        ))
        }
    }
}

impl StemmingConfig {
    pub(crate) fn to_optional(&self) -> Option<Algorithm> {
        match self {
            StemmingConfig::None => None,
            StemmingConfig::Arabic => Some(Algorithm::Arabic),
            StemmingConfig::Danish => Some(Algorithm::Danish),
            StemmingConfig::Dutch => Some(Algorithm::Dutch),
            StemmingConfig::English => Some(Algorithm::English),
            StemmingConfig::Finnish => Some(Algorithm::Finnish),
            StemmingConfig::French => Some(Algorithm::French),
            StemmingConfig::German => Some(Algorithm::German),
            StemmingConfig::Greek => Some(Algorithm::Greek),
            StemmingConfig::Hungarian => Some(Algorithm::Hungarian),
            StemmingConfig::Italian => Some(Algorithm::Italian),
            StemmingConfig::Norwegian => Some(Algorithm::Norwegian),
            StemmingConfig::Portuguese => Some(Algorithm::Portuguese),
            StemmingConfig::Romanian => Some(Algorithm::Romanian),
            StemmingConfig::Russian => Some(Algorithm::Russian),
            StemmingConfig::Spanish => Some(Algorithm::Spanish),
            StemmingConfig::Swedish => Some(Algorithm::Swedish),
            StemmingConfig::Tamil => Some(Algorithm::Tamil),
            StemmingConfig::Turkish => Some(Algorithm::Turkish),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_none_lowercase() {
        assert_eq!(
            StemmingConfig::from_str("none").unwrap(),
            StemmingConfig::None
        );
    }

    #[test]
    fn test_none_capital() {
        assert_eq!(
            StemmingConfig::from_str("None").unwrap(),
            StemmingConfig::None
        );
    }

    #[test]
    fn test_dutch() {
        assert_eq!(
            StemmingConfig::from_str("Dutch").unwrap(),
            StemmingConfig::Dutch
        );
    }

    #[test]
    fn test_dutch_lowercase() {
        assert_eq!(
            StemmingConfig::from_str("dutch").unwrap(),
            StemmingConfig::Dutch
        );
    }

    #[test]
    fn test_error() {
        assert!(StemmingConfig::from_str("Blorp").is_err());
    }

    #[test]
    fn test_dutch_tostring() {
        assert_eq!(StemmingConfig::Dutch.to_string(), "Dutch".to_string());
    }

    #[test]
    fn test_none_tostring() {
        assert_eq!(StemmingConfig::None.to_string(), "None".to_string());
    }
}
