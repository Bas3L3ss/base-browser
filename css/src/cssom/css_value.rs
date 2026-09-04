use crate::cssom::component_value::ComponentValue;

#[derive(Debug, Clone, PartialEq)]
pub struct CssValue {
    pub components: Vec<ComponentValue>,
    pub is_important: bool,
}
