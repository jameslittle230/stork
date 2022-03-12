use rust_stemmers::Algorithm;
use serde::{Deserialize, Serialize};
use std::convert::{From, TryFrom};
use std::fmt::Write;
use toml::Value;

#[derive(Serialize, Debug, Clone, PartialEq)]
#[serde(into = "String")]
#[serde(try_from = "String")]
pub enum StemmingConfig {
    None,
    Language(Algorithm),
}

impl Default for StemmingConfig {
    fn default() -> Self {
        StemmingConfig::Language(Algorithm::English)
    }
}

impl TryFrom<&String> for StemmingConfig {
    type Error = toml::de::Error;
    fn try_from(value: &String) -> Result<Self, Self::Error> {
        #[derive(Deserialize, Debug)]
        struct TempAlgStructure {
            lang: Algorithm,
        }

        if value == "none" || value == "None" {
            return Ok(StemmingConfig::None);
        }

        toml::from_str(format!("lang = \"{}\"", value).as_str())
            .map(|t: TempAlgStructure| StemmingConfig::Language(t.lang))
    }
}

impl<'de> serde::Deserialize<'de> for StemmingConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;

        if let Ok(Value::String(string)) = Deserialize::deserialize(deserializer) {
            StemmingConfig::try_from(&string).map_err(|_e| {
                serde::de::Error::custom(format!("Unexpected value `{}`, expected `none` or a language supported by https://snowballstem.org/, e.g. `Dutch`", string.clone()))
            })
        } else {
            Err(Error::custom(
            "Unexpected stemming config value; could not parse as string. (Maybe you need quotes?)",
        ))
        }
    }
}

impl From<StemmingConfig> for String {
    fn from(stemming_config: StemmingConfig) -> Self {
        let mut output = String::new();
        let _result = match stemming_config {
            StemmingConfig::Language(l) => write!(&mut output, "{:?}", l),
            StemmingConfig::None => write!(&mut output, "none"),
        };
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;
    #[test]
    fn test_none_lowercase() {
        assert_eq!(
            StemmingConfig::try_from(&"none".to_string()).unwrap(),
            StemmingConfig::None
        );
    }

    #[test]
    fn test_none_capital() {
        assert_eq!(
            StemmingConfig::try_from(&"None".to_string()).unwrap(),
            StemmingConfig::None
        );
    }

    #[test]
    fn test_dutch() {
        assert_eq!(
            StemmingConfig::try_from(&"Dutch".to_string()).unwrap(),
            StemmingConfig::Language(Algorithm::Dutch)
        );
    }

    #[test]
    fn test_error() {
        assert!(StemmingConfig::try_from(&"Blorp".to_string()).is_err())
    }

    #[test]
    fn test_dutch_tostring() {
        assert_eq!(
            // StemmingConfig::try_from(&"Dutch".to_string()).unwrap(),
            // StemmingConfig::Language(Algorithm::Dutch)
            String::from(StemmingConfig::Language(Algorithm::Dutch)),
            "Dutch".to_string()
        );
    }

    #[test]
    fn test_none_tostring() {
        assert_eq!(
            // StemmingConfig::try_from(&"Dutch".to_string()).unwrap(),
            // StemmingConfig::Language(Algorithm::Dutch)
            String::from(StemmingConfig::None),
            "none".to_string()
        );
    }
}
