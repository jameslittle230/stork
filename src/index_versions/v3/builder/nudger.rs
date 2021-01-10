use crate::config::Config;

/**
 * Nudge users to build better config files.
 *
 * Config files have to be backwards compatible, so we can't remove any fields
 * that the user might want to deserialize. But we _can_ detect that those
 * fields are being used, ignore them, and throw up a warning saying they're
 * being ignored.
 */

#[derive(Debug, PartialEq)]
pub(super) struct Nudger {
    nudges: Vec<Nudge>,
}

#[derive(Debug, PartialEq)]
enum Nudge {
    InputSurroundingWordCount,
}

impl Nudge {
    fn description(&self) -> &str {
        match self {
            Nudge::InputSurroundingWordCount => "`input.surrounding_word_count` is deprecated and has no effect. Please use output.excerpt_buffer instead."
        }
    }
}

impl From<&Config> for Nudger {
    fn from(config: &Config) -> Self {
        let mut nudges: Vec<Nudge> = vec![];

        if config.input.UNUSED_surrounding_word_count.is_some() {
            nudges.push(Nudge::InputSurroundingWordCount)
        }

        Nudger { nudges }
    }
}

impl Nudger {
    pub(super) fn is_empty(&self) -> bool {
        return self.nudges.is_empty();
    }

    pub(super) fn generate_formatted_output(&self) -> String {
        let mut output: String = "".to_string();

        if !&self.nudges.is_empty() {
            output.push_str("== Config Warnings ==");
        }

        for nudge in &self.nudges {
            output.push('\n');
            output.push_str(nudge.description())
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::*;

    #[test]
    fn create_nudge() {
        let intended = Nudger {
            nudges: vec![Nudge::InputSurroundingWordCount],
        };

        let generated = Nudger::from(&Config {
            input: InputConfig {
                UNUSED_surrounding_word_count: Some(12),
                ..Default::default()
            },
            output: OutputConfig::default(),
        });

        assert_eq!(intended, generated)
    }

    #[test]
    fn default_config_creates_empty_nudge() {
        let intended = Nudger { nudges: vec![] };
        let generated = Nudger::from(&Config::default());
        assert_eq!(intended, generated)
    }

    #[test]
    fn nudge_description() {
        let intended = "== Config Warnings ==\n`input.surrounding_word_count` is deprecated and has no effect. Please use output.excerpt_buffer instead."
            .to_string();

        let generated = Nudger {
            nudges: vec![Nudge::InputSurroundingWordCount],
        }
        .generate_formatted_output();

        assert_eq!(intended, generated)
    }
}
