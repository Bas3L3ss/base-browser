use crate::cssom::component_value::ComponentValue;

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    name: String,
    args: Vec<ComponentValue>,
}
