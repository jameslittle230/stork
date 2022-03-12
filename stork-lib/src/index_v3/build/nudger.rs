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
    OutputFile,
}

impl Nudge {
    fn description(&self) -> &str {
        match self {
            Nudge::InputSurroundingWordCount => "The config option `input.surrounding_word_count` is deprecated and has no effect. Please use output.excerpt_buffer instead.",
            Nudge::OutputFile => "The config option `output.filename` is deprecated and has no effect. Please use the --output command line option instead."
        }
    }
}

impl From<&Config> for Nudger {
    fn from(config: &Config) -> Self {
        let mut nudges: Vec<Nudge> = vec![];

        if config.input.UNUSED_surrounding_word_count.is_some() {
            nudges.push(Nudge::InputSurroundingWordCount)
        }

        if config.output.UNUSED_filename.is_some() {
            nudges.push(Nudge::OutputFile)
        }

        Nudger { nudges }
    }
}

impl Nudger {
    pub(super) fn print(&self) {
        if !self.nudges.is_empty() {
            eprintln!("Config Warnings:");
        }

        for nudge in &self.nudges {
            eprintln!("{}", nudge.description());
        }
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
}
