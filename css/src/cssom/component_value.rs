use crate::cssom::function::Function;

#[derive(Debug, Clone, PartialEq)]
pub enum ComponentValue {
    // Keywords/identifiers
    Ident(String),

    // "Hello"
    String(String),

    // #fff, #ffffff
    Hash(String),

    // 123
    Number(f32),

    // 10px, 1fr, 2em, 3rem...
    Dimension { value: f32, unit: String },

    // 50%
    Percentage(f32),

    // rgb(...), calc(...), var(...), url(...)
    Function(Function),

    // Used by shorthand properties
    Delim(char),
}
