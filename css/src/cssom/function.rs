use crate::cssom::component_value::ComponentValue;

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: String,
    pub args: Vec<ComponentValue>,
}

impl Function {
    pub fn new(name: String, args: Vec<ComponentValue>) -> Self {
        Function { name, args }
    }
}
