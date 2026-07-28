#[derive(Debug, PartialEq, Clone)]
pub enum CssTokenizerError {
    UnexpectedChar(char, usize),
    UnterminatedString(usize),
    UnterminatedComment(usize),
}

#[derive(Debug, PartialEq, Clone)]
pub enum CssToken {
    Ident(String), // e.g. Ident("color")
    Number(f32),   // e.g. Number(12.5)
    Dimension {
        // e.g. Dimension { value: 10.0, unit: "px".to_string() }
        value: f32,
        unit: String,
    },
    Percentage(f32), // e.g. Percentage(50.0)  // for "50%"
    String(String),  // e.g. String("hello world")
    Hash(String),    // e.g. Hash("abc123")

    Colon,      // e.g. ':'
    Semicolon,  // e.g. ';'
    LeftBrace,  // e.g. '{'
    RightBrace, // e.g. '}'
    LeftParen,  // e.g. '('
    RightParen, // e.g. ')'
    Comma,      // e.g. ','

    Whitespace, // e.g. ' ', '\t', '\n'

    AtKeyword(String), // e.g. AtKeyword("media")

    // General delimiter variant for single-character non-special tokens.
    Delim(char), // e.g. Delim('$')

    // Example: explicit variants for common delimiters/operators enhance pattern matching ergonomics
    Star,         // '*' in universal or attribute selectors — e.g. in "*", or "[attr*=value]"
    Plus,         // '+' in adjacent sibling combinator — e.g. "div + span"
    GreaterThan,  // '>' in child combinator — e.g. "ul > li"
    Tilde,        // '~' in general sibling combinator — e.g. "div ~ p"
    Dot,          // '.' in class selectors — e.g. ".foo"
    OpenBracket,  // '[' in attribute selectors — e.g. "[attr]"
    CloseBracket, // ']' in attribute selectors — e.g. "[attr]"
    Equals,       // '=' in attribute selectors — e.g. "[attr=value]"

    Error(CssTokenizerError), // e.g. Error(CssTokenizerError::UnexpectedChar('#', 3))
    EOF,                      // End of file/input
}
