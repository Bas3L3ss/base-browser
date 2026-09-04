use crate::CssValue;

#[derive(Debug, Clone, PartialEq)]
pub enum Overflow {
    Visible,
    Hidden,
    Scroll,
    Auto,
    Clip,
}

impl Overflow {
    pub fn from_css(value: &CssValue) -> Self {
        // Try to extract the identifier from one of the value's component values.
        if let Some(ident) = value.components.first().and_then(|c| c.as_ident()) {
            match ident {
                "visible" => Overflow::Visible,
                "hidden" => Overflow::Hidden,
                "scroll" => Overflow::Scroll,
                "auto" => Overflow::Auto,
                "clip" => Overflow::Clip,
                _ => Overflow::Visible, // fallback; 'visible' is the initial value in CSS
            }
        } else {
            Overflow::Visible
        }
    }
}
