use std::{collections::HashMap, format};

use crate::{
    ComponentValue,
    cssom::{css_value::CssValue, property::Property},
};

#[derive(Debug, Clone, PartialEq)]
pub enum Display {
    Block,
    InlineBlock,
    Inline,
    None,
    Flex,
    Grid,
    ListItem,
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
                        "inline-block" => Display::InlineBlock,
                        "none" => Display::None,
                        "flex" => Display::Flex,
                        "grid" => Display::Grid,
                        "list-item" => Display::ListItem,
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
