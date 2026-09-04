use crate::{CssValue, Length};

#[derive(Debug, Clone, PartialEq)]
pub enum MinLength {
    Auto,
    Length(Length),
}

impl MinLength {
    pub fn from_css(value: &CssValue) -> Self {
        if let Some(length) = value.components.first().and_then(|c| c.as_length()) {
            MinLength::Length(length)
        } else {
            MinLength::Auto
        }
    }
}

pub enum MaxLength {
    None,
    Length(Length),
}

impl MaxLength {
    pub fn from_css(value: &CssValue) -> Self {
        // In CSS, "none" is often used for max-width/max-height to mean "no maximum".
        if let Some(ident) = value.components.first().and_then(|c| c.as_ident()) {
            if ident.eq_ignore_ascii_case("none") {
                return MaxLength::None;
            }
        }
        if let Some(length) = value.components.first().and_then(|c| c.as_length()) {
            MaxLength::Length(length)
        } else {
            MaxLength::None
        }
    }
}
