use crate::{
    cssom::{declaration::Declaration, keyframe::Keyframe, selector::ComplexSelector},
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
    pub name: String,
    pub prelude: Vec<CssToken>,
    pub block: Option<AtRuleBlock>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AtRuleBlock {
    RuleList(Vec<CssRule>),
    DeclarationList(Vec<Declaration>),
    KeyframesRule(Vec<Keyframe>),
    Raw(Vec<CssToken>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum AtRuleKind {
    RuleList,
    DeclarationList,
    Unknown,
    KeyFrames,
}
