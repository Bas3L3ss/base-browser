use crate::cli::data_structure::Warning;

pub fn print_warning(warning: Warning) {
    match warning {
        Warning::DeprecatedOption(opt) => {
            eprintln!(
                "Warning: The option '{}' is deprecated and may be removed in future versions.",
                opt
            );
        }
        Warning::DuplicateOptionIgnored(opt) => {
            eprintln!(
                "Warning: Duplicate option '{}' encountered; the latter occurrence is ignored.",
                opt
            );
        }
        Warning::UnknownOptionEncountered(opt) => {
            eprintln!(
                "Warning: Unknown option '{}' encountered; it was ignored.",
                opt
            );
        }
        Warning::OptionValueTruncated { option, original } => {
            eprintln!(
                "Warning: The value for option '{}' was too long and has been truncated. Original: '{}'",
                option, original
            );
        }
        Warning::UnusedArgument(arg) => {
            eprintln!(
                "Warning: Unused argument '{}' detected; it was ignored.",
                arg
            );
        }
        Warning::OptionOverridden {
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
