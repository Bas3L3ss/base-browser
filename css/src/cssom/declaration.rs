use crate::cssom::{css_value::CssValue, property::Property};

pub struct Declaration {
    pub property: Property,
    pub value: CssValue,
}
