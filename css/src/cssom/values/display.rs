use crate::CssValue;

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

impl Display {
    pub fn from_css(value: &CssValue) -> Self {
        // Try to extract the identifier from one of the value's component values.
        if let Some(ident) = value.components.first().and_then(|c| c.as_ident()) {
            match ident {
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
            Display::Block
        }
    }
}
