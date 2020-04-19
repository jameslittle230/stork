use rust_stemmers::Algorithm;
use serde::{Deserialize, Serialize};
use std::fmt::{Write};
use toml::Value;

#[derive(Serialize, Debug, Clone)]
#[serde(into = "String")]
#[serde(try_from = "String")]
pub enum StemmingConfig {
    None,
    Language(Algorithm)
}

impl Default for StemmingConfig {
    fn default() -> Self {
        StemmingConfig::Language(Algorithm::English)
    }
}

impl<'de> serde::Deserialize<'de> for StemmingConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;

        if let Ok(Value::String(string)) = Deserialize::deserialize(deserializer) {
            if string == "none" || string == "None" {
                return Ok(StemmingConfig::None);
            }

            let mut w = String::new();
            let _ = write!(&mut w, "lang = \"{}\"", string);

            #[derive(Deserialize, Debug)]
            struct TempAlgStructure {
                lang: Algorithm,
            }

            let maybe_alg: Result<TempAlgStructure, toml::de::Error> = toml::from_str(&w);

            if let Ok(t) = maybe_alg {
                return Ok(StemmingConfig::Language(t.lang));
            }

            let mut error_msg = String::new();
            let _ = write!(&mut error_msg, "Unexpected value `{}`, expected `none` or a language supported by https://snowballstem.org/, e.g. `Dutch`", string);
            return Err(Error::custom(error_msg));
        }

        Err(Error::custom("Unexpected stemming config value; could not parse as string. (Maybe you need quotes?)"))
    }
}

impl From<StemmingConfig> for String {
    fn from(stemming_config: StemmingConfig) -> Self {
        let mut output = String::new();
        let _result = match stemming_config {
            StemmingConfig::Language(l) => write!(&mut output, "{:?}", l),
            StemmingConfig::None        => write!(&mut output, "none"),
        };
        output
    }
}
