use std::{collections::HashMap, format};

use crate::{
    ComponentValue,
    cssom::{css_value::CssValue, property::Property},
};

#[derive(Debug, Clone, PartialEq)]
pub enum Display {
    Block,
    Inline,
    None,
    Flex,
    Grid,
    Unknown(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ComputedStyle {
    pub properties: HashMap<Property, CssValue>,
}
impl ComputedStyle {
    pub fn display(&self) -> Display {
        if let Some(val) = self.properties.get(&Property::Display) {
            if val.components.len() == 1 {
                if let ComponentValue::Ident(ref s) = val.components[0] {
                    match s.as_str() {
                        "block" => Display::Block,
                        "inline" => Display::Inline,
                        "none" => Display::None,
                        "flex" => Display::Flex,
                        "grid" => Display::Grid,
                        other => Display::Unknown(other.to_string()),
                    }
                } else {
                    Display::Unknown(format!("{:?}", val.components[0]))
                }
            } else {
                Display::Unknown(format!("{:?}", val.components))
            }
        } else {
            // Shouldn't happen if UA stylesheet assigns display to every element
            Display::Unknown("no display".to_string())
        }
    }
}
