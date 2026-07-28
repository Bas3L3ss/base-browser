use crate::cssom::{declaration::Declaration, selector::ComplexSelector};
#[derive(Debug, Clone, PartialEq)]
pub struct Rule {
    pub selectors: Vec<ComplexSelector>,
    pub declarations: Vec<Declaration>,
}
