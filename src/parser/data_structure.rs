/// A container for the result of a parse operation.
///
/// # Type Parameters
/// - `T`: The type of the successfully parsed value.
/// - `W`: The type used for any warnings encountered during parsing.
pub struct ParsedOutput<T, W> {
    pub value: T,
    pub warnings: Vec<W>,
}

impl<T, W> ParsedOutput<T, W> {
    /// Constructs a new `ParsedOutput` from a parsed value and a list of warnings.
    ///
    /// # Parameters
    /// - `value`: The parsed result of type `T`.
    /// - `warnings`: A vector containing zero or more warnings of type `W`.
    pub fn new(value: T, warnings: Vec<W>) -> Self {
        Self { value, warnings }
    }
}

/// A trait for types that can be parsed from an input, optionally producing warnings.
///
/// # Type Parameters
/// - `W`: The type of warning that may be generated during parsing.
/// - `I`: The input type that this implementation parses.
pub trait Parse<W, I>: Sized {
    /// Parses a value from the provided input.
    ///
    /// # Parameters
    /// - `input`: The input of type `I` to parse.
    ///
    /// # Returns
    /// - `Ok(ParsedOutput<Self, W>)` if parsing succeeds (possibly with warnings).
    /// - `Err(anyhow::Error)` if parsing fails.
    fn parse(input: I) -> anyhow::Result<ParsedOutput<Self, W>>;
}
