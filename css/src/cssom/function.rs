use std::format;

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

impl ToString for Function {
    fn to_string(&self) -> String {
        let args_str = self
            .args
            .iter()
            .map(|arg| arg.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        format!("{}({})", self.name, args_str)
    }
}
