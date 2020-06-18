use super::{ExitCode, EXIT_FAILURE, EXIT_SUCCESS};
use std::fmt;

pub struct Argparse {
    commands: Vec<Command>,
    help_text: Option<String>,
}

struct Command {
    name: String,
    action: fn(&[String]),
    number_of_args: ValueOrRange,
}

enum ValueOrRange {
    Value(u8),
    Range(u8, u8),
}

impl fmt::Display for ValueOrRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValueOrRange::Value(val) => write!(f, "{}", val),

            ValueOrRange::Range(min, max) => write!(f, "between {} and {}", min, max),
        }
    }
}

impl Argparse {
    pub fn new() -> Argparse {
        Argparse {
            commands: vec![],
            help_text: None,
        }
    }

    pub fn register(&mut self, cmd_name: &str, action: fn(&[String]), number_of_args: u8) {
        self.commands.push(Command {
            name: cmd_name.to_string(),
            action,
            number_of_args: ValueOrRange::Value(number_of_args),
        })
    }

    #[allow(dead_code)]
    pub fn register_range(&mut self, cmd_name: &str, action: fn(&[String]), args_range: (u8, u8)) {
        let min = std::cmp::min(args_range.0, args_range.1);
        let max = std::cmp::max(args_range.0, args_range.1);
        let number_of_args = if min == max {
            ValueOrRange::Value(min)
        } else {
            ValueOrRange::Range(min, max)
        };

        self.commands.push(Command {
            name: cmd_name.to_string(),
            action,
            number_of_args,
        })
    }

    pub fn register_help(&mut self, text: &str) {
        self.help_text = Some(text.to_string());
    }

    pub fn exec(&self, args: Vec<String>) -> ExitCode {
        if args.len() < 2 || ["-h", "--help"].contains(&args[1].as_str()) {
            if let Some(help_text) = &self.help_text {
                println!("{}", help_text);
                return EXIT_SUCCESS;
            }
        }

        for command in &self.commands {
            if args[1] == ["--", &command.name].concat() {
                let number_of_args = args.len() - 2;
                let valid = match command.number_of_args {
                    ValueOrRange::Value(val) => (number_of_args as u8) == val,

                    ValueOrRange::Range(min, max) => {
                        (number_of_args as u8) >= min && (number_of_args as u8) <= max
                    }
                };

                if !valid {
                    println!(
                        "Wrong number of arguments given to `{}` command. Expected {} but got {}.",
                        command.name, command.number_of_args, number_of_args
                    );
                    return EXIT_FAILURE;
                } else {
                    (command.action)(&args);
                    return EXIT_SUCCESS;
                }
            }
        }

        println!("Command not found: `{}`.", args[1]);
        EXIT_FAILURE
    }
}
