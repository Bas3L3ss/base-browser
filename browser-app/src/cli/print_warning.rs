use crate::cli::cli_warning::CliWarning;

pub fn print_warning(warning: &CliWarning) {
    match warning {
        CliWarning::DeprecatedOption(opt) => {
            eprintln!(
                "Warning: The option '{}' is deprecated and may be removed in future versions.",
                opt
            );
        }
        CliWarning::DuplicateOptionIgnored(opt) => {
            eprintln!(
                "Warning: Duplicate option '{}' encountered; the latter occurrence is ignored.",
                opt
            );
        }
        CliWarning::UnknownOptionEncountered(opt) => {
            eprintln!(
                "Warning: Unknown option '{}' encountered; it was ignored.",
                opt
            );
        }
        CliWarning::OptionValueTruncated { option, original } => {
            eprintln!(
                "Warning: The value for option '{}' was too long and has been truncated. Original: '{}'",
                option, original
            );
        }
        CliWarning::UnusedArgument(arg) => {
            eprintln!(
                "Warning: Unused argument '{}' detected; it was ignored.",
                arg
            );
        }
        CliWarning::OptionOverridden {
            option,
            previous_value,
            new_value,
        } => {
            eprintln!(
                "Warning: Option '{}' was set multiple times. Previous value: '{}', new value: '{}'. Using latest.",
                option, previous_value, new_value
            );
        }
    }
}
