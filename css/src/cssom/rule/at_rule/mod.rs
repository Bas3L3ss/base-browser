pub mod import;
pub use import::*;

pub mod layout;
pub use layout::*;

pub mod font_face;
pub use font_face::*;

pub mod media_query;
pub use media_query::*;

use crate::{CssToken, Declaration, Keyframe, rule::rule::CssRule};

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

#[derive(Debug, Clone, PartialEq, Default)]
pub struct AtRule {
    pub name: String,
    pub prelude: Vec<CssToken>,
    pub block: Option<AtRuleBlock>,
}
