use anyhow::{Result, bail};

use crate::{
    cli::{
        cli_warning::CliWarning,
        utils::{is_accepted_arg, is_accepted_subcommand},
    },
    parser::{generic_struct::ParsedOutput, traits::Parse},
};

#[derive(Debug, Clone)]
pub struct Options {
    pub url: Option<String>,
}

impl Options {
    pub fn new() -> Options {
        Options { url: None }
    }
}

#[derive(Debug)]
pub struct Command {
    pub opts: Options,
    pub sub_command: Option<String>,
}

impl Command {
    pub fn new(opts: Options, sub_command: Option<String>) -> Command {
        Command { opts, sub_command }
    }
}

impl Parse<CliWarning, Vec<String>> for Command {
    fn parse(args: Vec<String>) -> Result<ParsedOutput<Command, CliWarning>> {
        let mut warnings: Vec<CliWarning> = Vec::new();
        let mut opts: Options = Options::new();
        let mut sub_command: Option<String> = None;

        let mut i = 1;
        while i < args.len() {
            let arg = args[i].as_str();

            if let Some(needs_value) = is_accepted_arg(arg) {
                // Handle accepted argument (e.g., --url)
                if needs_value {
                    if i + 1 < args.len() {
                        if arg == "--url" {
                            if opts.url.is_none() {
                                opts.url = Some(args[i + 1].clone());
                            } else {
                                warnings.push(CliWarning::OptionOverridden {
                                    option: arg.to_string(),
                                    previous_value: opts.url.unwrap().clone(),
                                    new_value: args[i + 1].clone(),
                                });
                                opts.url = Some(args[i + 1].clone());
                            }
                            i += 2;
                            continue;
                        }
                    } else {
                        bail!("Missing value for option '{}'", arg);
                    }
                }
                i += 1;
                continue;
            } else if is_accepted_subcommand(arg) {
                sub_command = Some(arg.to_string());
                i += 1;
                continue;
            } else if !arg.starts_with("--") && opts.url.is_none() {
                // Accept positional URL if --url wasn't set yet
                opts.url = Some(arg.to_string());
                i += 1;
                continue;
            } else if !arg.starts_with("--") {
                // Unused argument
                warnings.push(CliWarning::UnusedArgument(arg.to_string()));
                i += 1;
                continue;
            } else {
                warnings.push(CliWarning::UnknownOptionEncountered(arg.to_string()));
                i += 1;
                continue;
            }
        }

        Ok(ParsedOutput::new(Command::new(opts, sub_command), warnings))
    }
}
