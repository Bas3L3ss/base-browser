use crate::{
    cssom::{component_value::ComponentValue, declaration::Declaration, selector::ComplexSelector},
    tokenizer::css_token::CssToken,
};
#[derive(Debug, Clone, PartialEq)]
pub enum CssRule {
    Style(StyleRule),
    At(AtRule),
}

#[derive(Debug, Clone, PartialEq)]
pub struct StyleRule {
    pub selectors: Vec<ComplexSelector>,
    pub declarations: Vec<Declaration>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AtRule {
    pub rule_name: String,
    pub preludes: Vec<ComponentValue>,
    pub block: Option<AtRuleBlock>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AtRuleBlock {
    RuleList(Vec<CssRule>),
    DeclarationList(Vec<Declaration>),
    Raw(Vec<CssToken>),
}
