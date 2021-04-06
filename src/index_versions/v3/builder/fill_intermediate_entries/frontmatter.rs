use crate::common::Fields;
use crate::config::FrontmatterConfig;
use frontmatter::{parse_and_find_content, Yaml};
use std::collections::HashMap;

pub fn parse_frontmatter(handling: &FrontmatterConfig, buffer: &str) -> (Fields, Box<String>) {
    let default_output = (HashMap::new(), Box::new(buffer.to_string()));
    match handling {
        FrontmatterConfig::Ignore => default_output,
        FrontmatterConfig::Omit => {
            if let Ok((_yaml, text)) = parse_and_find_content(&buffer) {
                (HashMap::new(), Box::new(text.trim().to_string()))
            } else {
                default_output
            }
        }
        FrontmatterConfig::Parse => {
            if let Ok((Some(Yaml::Hash(map)), text)) = parse_and_find_content(&buffer) {
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

            default_output
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn omit_option() {
        let expected: (Fields, String) = (HashMap::new(), "this is not".to_string());
        let output = parse_frontmatter(
            &FrontmatterConfig::Omit,
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
    fn parse_option() {
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
            &FrontmatterConfig::Parse,
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
