use crate::cssom::{css_value::CssValue, property::Property};
#[derive(Debug, Clone, PartialEq)]

pub struct Declaration {
    pub property: Property,
    pub value: CssValue,
}
