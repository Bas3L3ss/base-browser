use crate::cssom::{function::Function, unit::Unit};

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
    Dimension { value: f32, unit: Unit },

    // 50%
    Percentage(f32),

    // rgb(...), calc(...), var(...), url(...)
    Function(Function),

    // Used by shorthand properties
    Delim(char),
}

impl ToString for ComponentValue {
    fn to_string(&self) -> String {
        match self {
            ComponentValue::Ident(ident) => ident.clone(),
            ComponentValue::String(s) => format!("\"{}\"", s),
            ComponentValue::Hash(hash) => format!("#{}", hash),
            ComponentValue::Number(n) => format!("{}", n),
            ComponentValue::Dimension { value, unit } => format!("{}{}", value, unit.to_string()),
            ComponentValue::Percentage(p) => format!("{}%", p),
            ComponentValue::Delim(c) => c.to_string(),
            ComponentValue::Function(f) => f.to_string(),
        }
    }
}
