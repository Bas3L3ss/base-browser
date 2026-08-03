use std::matches;

use crate::cssom::component_value::ComponentValue;

#[derive(Debug, Clone, PartialEq)]
pub struct CssValue {
    pub components: Vec<ComponentValue>,
    pub is_important: bool,
}

impl CssValue {
    pub fn is_inherit(&self) -> bool {
        self.components.len() == 1
            && matches!(
                self.components[0],
                ComponentValue::Ident(ref s) if s == "inherit"
            )
    }
}
