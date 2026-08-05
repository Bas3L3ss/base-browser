use std::collections::HashMap;

use crate::{
    AspectRatio, BoxSizing, Clear, ComponentValue, CssValue, Edges, Float, FontSize, Length,
    MaxLength, MinLength, Overflow, Position, Property, Unit, Visibility, display::Display,
};

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
        Edges {
            top: self
                .properties
                .get(&Property::MarginTop)
                .and_then(|v| v.components.first())
                .and_then(|v| v.as_length())
                .unwrap_or(Length::Auto),
            right: self
                .properties
                .get(&Property::MarginRight)
                .and_then(|v| v.components.first())
                .and_then(|v| v.as_length())
                .unwrap_or(Length::Auto),
            bottom: self
                .properties
                .get(&Property::MarginBottom)
                .and_then(|v| v.components.first())
                .and_then(|v| v.as_length())
                .unwrap_or(Length::Auto),
            left: self
                .properties
                .get(&Property::MarginLeft)
                .and_then(|v| v.components.first())
                .and_then(|v| v.as_length())
                .unwrap_or(Length::Auto),
        }
    }

    pub fn padding(&self) -> Edges<Length> {
        Edges {
            top: self
                .properties
                .get(&Property::PaddingTop)
                .and_then(|v| v.components.first())
                .and_then(|v| v.as_length())
                .unwrap_or(Length::Auto),
            right: self
                .properties
                .get(&Property::PaddingRight)
                .and_then(|v| v.components.first())
                .and_then(|v| v.as_length())
                .unwrap_or(Length::Auto),
            bottom: self
                .properties
                .get(&Property::PaddingBottom)
                .and_then(|v| v.components.first())
                .and_then(|v| v.as_length())
                .unwrap_or(Length::Auto),
            left: self
                .properties
                .get(&Property::PaddingLeft)
                .and_then(|v| v.components.first())
                .and_then(|v| v.as_length())
                .unwrap_or(Length::Auto),
        }
    }

    pub fn border_width(&self) -> Edges<Length> {
        fn border_width_or_medium(val: Option<&CssValue>) -> Length {
            val.and_then(|v| v.components.first())
                .and_then(|v| v.as_length())
                .unwrap_or(Length::Dimension {
                    value: 3.0,
                    unit: Unit::Px,
                })
        }

        Edges {
            top: border_width_or_medium(self.properties.get(&Property::BorderTopWidth)),
            right: border_width_or_medium(self.properties.get(&Property::BorderRightWidth)),
            bottom: border_width_or_medium(self.properties.get(&Property::BorderBottomWidth)),
            left: border_width_or_medium(self.properties.get(&Property::BorderLeftWidth)),
        }
    }

    pub fn min_max_width(&self) -> (MinLength, MaxLength) {
        let min = self
            .properties
            .get(&Property::MinWidth)
            .map(MinLength::from_css)
            .unwrap_or(MinLength::Auto);

        let max = self
            .properties
            .get(&Property::MaxWidth)
            .map(MaxLength::from_css)
            .unwrap_or(MaxLength::None);

        (min, max)
    }

    pub fn min_max_height(&self) -> (MinLength, MaxLength) {
        let min = self
            .properties
            .get(&Property::MinHeight)
            .map(MinLength::from_css)
            .unwrap_or(MinLength::Auto);

        let max = self
            .properties
            .get(&Property::MaxHeight)
            .map(MaxLength::from_css)
            .unwrap_or(MaxLength::None);

        (min, max)
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

    pub fn box_sizing(&self) -> BoxSizing {
        self.properties
            .get(&Property::BoxSizing)
            .and_then(|v| v.components.first())
            .and_then(|component| match component.as_ident() {
                Some("content-box") => Some(BoxSizing::ContentBox),
                Some("border-box") => Some(BoxSizing::BorderBox),
                _ => None,
            })
            .unwrap_or(BoxSizing::BorderBox)
    }

    pub fn position(&self) -> Position {
        self.properties
            .get(&Property::Position)
            .map(Position::from_css)
            .unwrap_or(Position::Static)
    }

    pub fn offset(&self) -> (Length, Length, Length, Length) {
        let left = self
            .properties
            .get(&Property::Left)
            .and_then(|v| v.components.first())
            .and_then(|component| component.as_length())
            .unwrap_or(Length::Auto);
        let top = self
            .properties
            .get(&Property::Top)
            .and_then(|v| v.components.first())
            .and_then(|component| component.as_length())
            .unwrap_or(Length::Auto);
        let right = self
            .properties
            .get(&Property::Right)
            .and_then(|v| v.components.first())
            .and_then(|component| component.as_length())
            .unwrap_or(Length::Auto);
        let bottom = self
            .properties
            .get(&Property::Bottom)
            .and_then(|v| v.components.first())
            .and_then(|component| component.as_length())
            .unwrap_or(Length::Auto);
        (left, top, right, bottom)
    }

    pub fn font_size(&self) -> Option<Length> {
        self.properties
            .get(&Property::FontSize)
            .and_then(|css_value| FontSize::from_css(css_value))
    }

    pub fn z_index(&self) -> Option<i32> {
        self.properties
            .get(&Property::ZIndex)
            .and_then(|v| v.components.first())
            .and_then(|component| match component {
                ComponentValue::Number(num) => Some(*num as i32),
                ComponentValue::Ident(ident) if ident == "auto" => None,
                _ => None,
            })
    }

    pub fn float(&self) -> Float {
        self.properties
            .get(&Property::Float)
            .map(Float::from_css)
            .unwrap_or(Float::None)
    }

    pub fn clear(&self) -> Clear {
        self.properties
            .get(&Property::Clear)
            .map(Clear::from_css)
            .unwrap_or(Clear::None)
    }

    pub fn aspect_ratio(&self) -> AspectRatio {
        self.properties
            .get(&Property::AspectRatio)
            .map(AspectRatio::from_css)
            .unwrap_or(AspectRatio::Auto)
    }

    pub fn visibility(&self) -> Visibility {
        self.properties
            .get(&Property::Visibility)
            .map(Visibility::from_css)
            .unwrap_or(Visibility::Visible)
    }

    pub fn overflow(&self) -> Overflow {
        self.properties
            .get(&Property::Overflow)
            .map(Overflow::from_css)
            .unwrap_or(Overflow::Visible)
    }

    fn get(&self, property: &Property) -> Option<&CssValue> {
        self.properties.get(property)
    }
}
