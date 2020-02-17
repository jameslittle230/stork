use super::{ExitCode, EXIT_FAILURE, EXIT_SUCCESS};

struct Command {
    name: String,
    action: fn(&[String]),
}

pub struct Argparse {
    commands: Vec<Command>,
    help_text: Option<String>,
}

impl Argparse {
    pub fn new() -> Argparse {
        Argparse {
            commands: vec![],
            help_text: None,
        }
    }

    pub fn register(&mut self, cmd_name: &str, action: fn(&[String])) {
        self.commands.push(Command {
            name: cmd_name.to_string(),
            action,
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
                (command.action)(&args);
                return EXIT_SUCCESS;
            }
        }

        println!("Command not found: `{}`.", args[1]);
        EXIT_FAILURE
    }
}
