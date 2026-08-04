use crate::{ComponentValue, CssValue, Length, Unit};

pub struct FontSize;

impl FontSize {
    pub fn from_css(value: &CssValue) -> Option<Length> {
        if value.components.len() != 1 {
            return None;
        }
        match &value.components[0] {
            ComponentValue::Dimension { value, unit } => Some(Length::Dimension {
                value: *value,
                unit: unit.clone(),
            }),
            ComponentValue::Number(value) => Some(Length::Dimension {
                value: *value,
                unit: Unit::Px,
            }),
            ComponentValue::Ident(ident) => match ident.as_str() {
                "small" => Some(Length::Dimension {
                    value: 13.333, // px equivalent, common user-agent default
                    unit: Unit::Px,
                }),
                "medium" => Some(Length::Dimension {
                    value: 16.0, // px equivalent, common user-agent default
                    unit: Unit::Px,
                }),
                "large" => Some(Length::Dimension {
                    value: 18.0, // px equivalent, common user-agent default
                    unit: Unit::Px,
                }),
                "x-large" => Some(Length::Dimension {
                    value: 24.0, // px equivalent, common user-agent default
                    unit: Unit::Px,
                }),
                _ => None,
            },
            _ => None,
        }
    }
}
