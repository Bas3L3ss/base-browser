use std::env;

use anyhow::{Result, bail};

use crate::cli::{
    data_structure::{Options, ParsedOutput, Warning},
    utils::{is_accepted_arg, is_accepted_subcommand},
};

pub fn parse_cli() -> Result<ParsedOutput> {
    let args: Vec<String> = env::args().collect();
    let mut warnings: Vec<Warning> = Vec::new();
    let mut opts: Options = Options::new();
    let mut sub_commands: Option<String> = None;

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
                            warnings.push(Warning::OptionOverridden {
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
            sub_commands = Some(arg.to_string());
            i += 1;
            continue;
        } else if !arg.starts_with("--") && opts.url.is_none() {
            // Accept positional URL if --url wasn't set yet
            opts.url = Some(arg.to_string());
            i += 1;
            continue;
        } else if !arg.starts_with("--") {
            // Unused argument
            warnings.push(Warning::UnusedArgument(arg.to_string()));
            i += 1;
            continue;
        } else {
            warnings.push(Warning::UnknownOptionEncountered(arg.to_string()));
            i += 1;
            continue;
        }
    }

    Ok(ParsedOutput::finalize(opts, warnings, sub_commands))
}
