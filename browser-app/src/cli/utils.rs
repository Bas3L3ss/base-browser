const ACCEPTED_ARGS: &[(&str, bool)] = &[("--url", true)];

const ACCEPTED_SUB_COMMANDS: &[&str] = &["help"];

pub fn is_accepted_arg(arg: &str) -> Option<bool> {
    ACCEPTED_ARGS.iter().find_map(
        |&(name, has_value)| {
            if arg == name { Some(has_value) } else { None }
        },
    )
}

pub fn is_accepted_subcommand(arg: &str) -> bool {
    ACCEPTED_SUB_COMMANDS.contains(&arg)
}
