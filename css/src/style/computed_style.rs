use std::collections::HashMap;

use crate::cssom::{css_value::CssValue, property::Property};

#[derive(Debug, Clone, PartialEq)]
pub struct ComputedStyle {
    properties: HashMap<Property, CssValue>,
}
