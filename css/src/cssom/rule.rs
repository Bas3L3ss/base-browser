use crate::cssom::{declaration::Declaration, selector::Selector};

pub struct Rule {
    pub selectors: Vec<Selector>,
    pub declarations: Vec<Declaration>,
}
