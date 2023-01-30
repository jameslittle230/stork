use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, SmartDefault)]
#[serde(deny_unknown_fields, default)]
pub struct HTMLConfig {
    #[default = false]
    pub save_nearest_id: bool,

    #[default = "h1"]
    pub title_selector: String,

    #[default(_code = "vec![\"main\".to_string()]")]
    #[serde(default)]
    pub included_selectors: Vec<String>,

    #[default(Default::default())]
    #[serde(default)]
    pub excluded_selectors: Vec<String>,
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::build_config::html::HTMLConfig;

    #[test]
    fn defaults() {
        let expected = HTMLConfig {
            save_nearest_id: false,
            title_selector: "h1".to_string(),
            included_selectors: vec!["main".to_string()],
            excluded_selectors: vec![],
        };
        assert_eq!(HTMLConfig::default(), expected);
    }
}
