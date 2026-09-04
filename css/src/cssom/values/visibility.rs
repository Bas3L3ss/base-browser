use crate::CssValue;

#[derive(Debug, Clone, PartialEq)]
pub enum Visibility {
    Visible,
    Hidden,
    Collapse,
}

impl Visibility {
    pub fn from_css(value: &CssValue) -> Self {
        // Try to extract the identifier from one of the value's component values.
        if let Some(ident) = value.components.first().and_then(|c| c.as_ident()) {
            match ident {
                "visible" => Visibility::Visible,
                "hidden" => Visibility::Hidden,
                "collapse" => Visibility::Collapse,
                _ => Visibility::Visible, // fallback; 'visible' is the initial value in CSS
            }
        } else {
            Visibility::Visible
        }
    }
}
