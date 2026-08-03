#[derive(Debug)]
pub enum CliWarning {
    DeprecatedOption(String),
    DuplicateOptionIgnored(String),
    UnknownOptionEncountered(String),
    OptionValueTruncated {
        option: String,
        original: String,
    },
    UnusedArgument(String),
    OptionOverridden {
        option: String,
        previous_value: String,
        new_value: String,
    },
}
