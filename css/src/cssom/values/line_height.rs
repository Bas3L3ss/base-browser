use crate::{ComponentValue, CssValue, Length, Unit};

pub enum LineHeight {
    Specified(Length),
    Normal,
}

impl LineHeight {
    pub fn from_css(value: &CssValue, font_size: f32) -> Self {
        if value.components.len() != 1 {
            return LineHeight::Normal;
        }
        match &value.components[0] {
            ComponentValue::Dimension { value, unit } => LineHeight::Specified(Length::Dimension {
                value: *value,
                unit: unit.clone(),
            }),
            ComponentValue::Number(number) => LineHeight::Specified(Length::Dimension {
                value: number * font_size,
                unit: Unit::Px,
            }),
            ComponentValue::Ident(ident) => {
                if ident == "normal" {
                    LineHeight::Normal
                } else {
                    LineHeight::Normal
                }
            }
            _ => LineHeight::Normal,
        }
    }
}
