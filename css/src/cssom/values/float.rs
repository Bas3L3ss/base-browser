use crate::CssValue;

#[derive(Debug, Clone, PartialEq)]
pub enum Float {
    Left,
    Right,
    None,
}

impl Float {
    pub fn from_css(value: &CssValue) -> Self {
        if let Some(ident) = value.components.first().and_then(|c| c.as_ident()) {
            match ident {
                "left" => Float::Left,
                "right" => Float::Right,
                "none" => Float::None,
                _ => Float::None,
            }
        } else {
            Float::None
        }
    }
}
