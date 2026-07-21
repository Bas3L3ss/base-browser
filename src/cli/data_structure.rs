#[derive(Debug)]
pub struct ParsedOutput {
    pub opts: Options,
    pub warnings: Vec<Warning>,
    pub sub_command: Option<String>,
}

impl ParsedOutput {
    pub fn finalize(
        opts: Options,
        warnings: Vec<Warning>,
        sub_command: Option<String>,
    ) -> ParsedOutput {
        ParsedOutput {
            opts,
            warnings,
            sub_command,
        }
    }
}

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
pub enum ParseError {
    UnknownOption(String),
    MissingValue(String),
    MissingRequired(String),
    InvalidValue {
        option: String,
        value: String,
        expected: String,
    },
}

#[derive(Debug)]
pub enum Warning {
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
