#[derive(Debug, PartialEq, Clone)]
pub enum CssTokenizerError {
    UnexpectedChar(char, usize),
    UnterminatedString(usize),
    UnterminatedComment(usize),
}

#[derive(Debug, PartialEq, Clone)]
pub enum CssToken {
    Ident(String),
    Number(f32),
    Dimension { value: f32, unit: String },
    Percentage(f32),
    String(String),
    Hash(String),

    Colon,
    Semicolon,
    LeftBrace,
    RightBrace,
    LeftParen,
    RightParen,
    Comma,

    Whitespace,

    Star,
    Plus,
    GreaterThan,
    Tilde,

    Dot,

    OpenBracket,
    CloseBracket,
    Equals,

    AtKeyword(String),

    Error(CssTokenizerError),
    EOF,
}
