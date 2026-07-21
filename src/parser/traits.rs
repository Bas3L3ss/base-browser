use crate::parser::generic_struct::ParsedOutput;

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
