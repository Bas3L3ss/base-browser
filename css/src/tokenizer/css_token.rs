use crate::cssom::unit::Unit;
#[derive(Debug)]
pub enum CssToken {
    Ident(String),

    Number(f32),

    Dimension { value: f32, unit: Unit },

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

    EOF,
}
