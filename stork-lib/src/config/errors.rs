use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigReadError {
    #[error("Recieved empty configuration string")]
    EmptyString,

    #[error("Cannot parse config as TOML. Stork recieved error: `{0}`")]
    UnparseableTomlInput(#[from] toml::de::Error),

    #[error("Cannot parse config as JSON. Stork recieved error: `{0}`")]
    UnparseableJsonInput(#[from] serde_json::Error),
}

impl PartialEq for ConfigReadError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::UnparseableTomlInput(l0), Self::UnparseableTomlInput(r0)) => l0 == r0,

            // default case also catches UnparseableJsonInput, which would otherwise look like
            // the TomlInput case above, except serde_json::Error doesn't impl PartialEq.
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn from_toml_error() {
        let expected = "Cannot parse config as TOML. Stork recieved error: `expected an equals, found an identifier at line 1 column 6`";
        let computed = toml::from_str::<()>("this is bad toml")
            .map_err(ConfigReadError::from)
            .unwrap_err()
            .to_string();
        assert_eq!(expected, computed);
    }

    #[test]
    fn partial_eq_json() {
        let json_error_one = serde_json::from_str::<()>("this is not json").unwrap_err();
        let json_error_two = serde_json::from_str::<()>("{[}").unwrap_err();

        let config_read_error_one = ConfigReadError::UnparseableJsonInput(json_error_one);
        let config_read_error_two = ConfigReadError::UnparseableJsonInput(json_error_two);

        assert_eq!(config_read_error_one, config_read_error_two);
    }
}
