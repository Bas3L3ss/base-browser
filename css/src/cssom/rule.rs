use crate::{
    cssom::{component_value::ComponentValue, declaration::Declaration, selector::ComplexSelector},
    tokenizer::css_token::CssToken,
};
#[derive(Debug, Clone, PartialEq)]
pub struct StyleRule {
    pub selectors: Vec<ComplexSelector>,
    pub declarations: Vec<Declaration>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AtRule {
    pub name: String,
    pub prelude: Vec<ComponentValue>,
    pub block: Option<AtRuleBlock>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AtRuleBlock {
    RuleList(Vec<CssRule>),
    DeclarationList(Vec<Declaration>),
    Raw(Vec<CssToken>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum CssRule {
    Style(StyleRule),
    At(AtRule),
}
