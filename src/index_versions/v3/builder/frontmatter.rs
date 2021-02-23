use crate::common::Fields;
use crate::config::{FrontmatterConfig, InputConfig};
use frontmatter::{parse_and_find_content, Yaml};
use std::collections::HashMap;

pub fn parse_frontmatter(config: &InputConfig, buffer: &str) -> (Fields, Box<String>) {
    let default_output = (HashMap::new(), Box::new(buffer.to_string()));
    match config.frontmatter_handling {
        FrontmatterConfig::Ignore => default_output,
        FrontmatterConfig::Omit => {
            if let Ok((_yaml, text)) = parse_and_find_content(&buffer) {
                (HashMap::new(), Box::new(text.trim().to_string()))
            } else {
                default_output
            }
        }
        FrontmatterConfig::Parse => {
            if let Ok((yaml, text)) = parse_and_find_content(&buffer) {
                if let Some(yaml) = yaml {
                    if let Yaml::Hash(map) = yaml {
                        let fields = map
                            .into_iter()
                            .map(|(k, v)| {
                                (
                                    k.into_string().unwrap_or_else(|| "".to_string()),
                                    v.clone().into_string().unwrap_or_else(|| {
                                        v.into_i64()
                                            .map_or("default".to_string(), |i| i.to_string())
                                    }),
                                )
                            })
                            .collect();
                        return (fields, Box::new(text.trim().to_string()));
                    }
                }
            }

            default_output
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_default_input_config_omits_frontmatter() {
        let expected: (Fields, String) = (HashMap::new(), "this is not".to_string());
        let output = parse_frontmatter(
            &InputConfig::default(),
            &mut r#"---
this: "is frontmatter"
"that takes": "multiple lines"
"and has": 22
"different formats": +INF
---

this is not
        "#
            .to_string(),
        );

        let computed = (output.0, output.1.to_string());
        assert_eq!(expected, computed)
    }

    #[test]
    fn test_parse_option_parses_correctly_frontmatter() {
        let expected: (Fields, String) = (
            [
                ("this".to_string(), "is frontmatter".to_string()),
                ("that takes".to_string(), "multiple lines".to_string()),
                ("and has".to_string(), "22".to_string()),
                ("different formats".to_string(), "+INF".to_string()),
            ]
            .iter()
            .cloned()
            .collect(),
            "this is not".to_string(),
        );
        let output = parse_frontmatter(
            &InputConfig {
                frontmatter_handling: FrontmatterConfig::Parse,
                ..Default::default()
            },
            &mut r#"---
this: "is frontmatter"
"that takes": "multiple lines"
"and has": 22
"different formats": +INF
---

this is not
        "#
            .to_string(),
        );

        let computed = (output.0, output.1.to_string());
        assert_eq!(expected, computed)
    }
}
