use std::{format, matches};

use crate::{
    Color, Length,
    cssom::{function::Function, unit::Unit},
};

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
            ComponentValue::String(s) => s.clone(),
            ComponentValue::Hash(hash) => format!("#{}", hash),
            ComponentValue::Number(n) => format!("{}", n),
            ComponentValue::Dimension { value, unit } => format!("{}{}", value, unit.to_string()),
            ComponentValue::Percentage(p) => format!("{}%", p),
            ComponentValue::Delim(c) => c.to_string(),
            ComponentValue::Function(f) => f.to_string(),
        }
    }
}

impl ComponentValue {
    pub fn as_ident(&self) -> Option<&str> {
        match self {
            ComponentValue::Ident(s) => Some(s.as_str()),
            _ => None,
        }
    }

    pub fn as_dimension(&self) -> Option<(f32, &Unit)> {
        match self {
            ComponentValue::Dimension { value, unit } => Some((*value, unit)),
            _ => None,
        }
    }

    pub fn as_percentage(&self) -> Option<f32> {
        match self {
            ComponentValue::Percentage(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_length(&self) -> Option<Length> {
        match self {
            ComponentValue::Dimension { value, unit } => Some(Length::Dimension {
                value: *value,
                unit: unit.clone(),
            }),
            ComponentValue::Percentage(v) => Some(Length::Percent(*v)),
            ComponentValue::Ident(s) => match s.as_str() {
                "auto" => Some(Length::Auto),
                "thin" => Some(Length::Dimension {
                    value: 1.0,
                    unit: Unit::Px,
                }),
                "medium" => Some(Length::Dimension {
                    value: 3.0,
                    unit: Unit::Px,
                }),
                "thick" => Some(Length::Dimension {
                    value: 5.0,
                    unit: Unit::Px,
                }),
                _ => None,
            },

            _ => None,
        }
    }

    pub fn as_color(&self) -> Option<Color> {
        match self {
            ComponentValue::Ident(name) => Some(Color::Named(name.clone())),
            ComponentValue::Hash(hex) => Some(Color::Hex(hex.clone())),
            ComponentValue::Function(func) => Color::parse_function(func),
            _ => None,
        }
    }

    pub fn as_delim(&self) -> Option<char> {
        match self {
            ComponentValue::Delim(c) => Some(*c),
            _ => None,
        }
    }

    pub fn as_number(&self) -> Option<f32> {
        match self {
            ComponentValue::Number(n) => Some(*n),
            _ => None,
        }
    }

    pub fn is_keyword(&self, keyword: &str) -> bool {
        matches!(self.as_ident(), Some(s) if s == keyword)
    }
}
