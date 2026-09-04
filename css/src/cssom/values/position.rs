use crate::CssValue;

#[derive(Debug, Clone, PartialEq)]
pub enum Position {
    Static,
    Relative,
    Absolute,
    Fixed,
    Sticky,
}

impl Position {
    pub fn from_css(value: &CssValue) -> Self {
        if let Some(ident) = value.components.first().and_then(|c| c.as_ident()) {
            match ident {
                "static" => Position::Static,
                "relative" => Position::Relative,
                "absolute" => Position::Absolute,
                "fixed" => Position::Fixed,
                "sticky" => Position::Sticky,
                _ => Position::Static,
            }
        } else {
            Position::Static
        }
    }
}
