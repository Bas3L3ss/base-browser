use std::{collections::HashMap, todo};

use crate::{CssValue, Edges, Length, Property, display::Display};

#[derive(Debug, Clone, PartialEq)]
pub struct ComputedStyle {
    pub properties: HashMap<Property, CssValue>,
}

impl ComputedStyle {
    pub fn display(&self) -> Display {
        self.get(&Property::Display)
            .map(Display::from_css)
            .unwrap_or(Display::Inline)
    }

    pub fn margin(&self) -> Edges<Length> {
        Edges::from_properties(
            &self.properties,
            Property::MarginTop,
            Property::MarginRight,
            Property::MarginBottom,
            Property::MarginLeft,
            Property::Margin,
            Length::Auto,
        )
    }

    pub fn padding(&self) -> Edges<Length> {
        Edges::from_properties(
            &self.properties,
            Property::PaddingTop,
            Property::PaddingRight,
            Property::PaddingBottom,
            Property::PaddingLeft,
            Property::Padding,
            Length::Auto, // Per spec, initial padding is 0, but for other edges it might differ
        )
    }

    pub fn border(&self) -> Edges<Length> {
        todo!()
    }

    pub fn min_width(&self) -> Length {
        self.properties
            .get(&Property::MinWidth)
            .and_then(|v| v.components.first())
            .and_then(|component| component.as_length())
            .unwrap_or(Length::Auto)
    }

    pub fn max_width(&self) -> Length {
        self.properties
            .get(&Property::MaxWidth)
            .and_then(|v| v.components.first())
            .and_then(|component| component.as_length())
            .unwrap_or(Length::Auto)
    }

    pub fn min_height(&self) -> Length {
        self.properties
            .get(&Property::MinHeight)
            .and_then(|v| v.components.first())
            .and_then(|component| component.as_length())
            .unwrap_or(Length::Auto)
    }

    pub fn max_height(&self) -> Length {
        self.properties
            .get(&Property::MaxHeight)
            .and_then(|v| v.components.first())
            .and_then(|component| component.as_length())
            .unwrap_or(Length::Auto)
    }

    pub fn width(&self) -> Length {
        self.properties
            .get(&Property::Width)
            .and_then(|v| v.components.first())
            .and_then(|component| component.as_length())
            .unwrap_or(Length::Auto)
    }

    pub fn height(&self) -> Length {
        self.properties
            .get(&Property::Height)
            .and_then(|v| v.components.first())
            .and_then(|component| component.as_length())
            .unwrap_or(Length::Auto)
    }

    fn get(&self, property: &Property) -> Option<&CssValue> {
        self.properties.get(property)
    }
}
