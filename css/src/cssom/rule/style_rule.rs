use crate::{ComplexSelector, Declaration};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct StyleRule {
    pub selectors: Vec<ComplexSelector>,
    pub declarations: Vec<Declaration>,
    // pub layer: Option<usize>,
}
