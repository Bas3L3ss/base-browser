use crate::CssValue;

#[derive(Debug, Clone, PartialEq)]
pub enum Clear {
    None,
    Left,
    Right,
    Both,
}

impl Clear {
    pub fn from_css(value: &CssValue) -> Self {
        if let Some(ident) = value.components.first().and_then(|c| c.as_ident()) {
            match ident {
                "left" => Clear::Left,
                "right" => Clear::Right,
                "both" => Clear::Both,
                "none" => Clear::None,
                _ => Clear::None,
            }
        } else {
            Clear::None
        }
    }
}
