use crate::{ComponentValue, CssValue};

pub struct FontFamily;

impl FontFamily {
    pub fn from_css(value: &CssValue) -> Vec<String> {
        value
            .components
            .iter()
            .filter_map(|component| {
                match component {
                    // Accept quoted or unquoted family names
                    ComponentValue::String(s) => Some(s.clone()),
                    ComponentValue::Ident(ident) => Some(ident.clone()),
                    _ => None,
                }
            })
            .collect()
    }
}
