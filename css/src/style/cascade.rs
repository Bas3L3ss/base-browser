use std::collections::HashMap;

use html::dom::{dom::Dom, node::Node};

use crate::{CssOm, Origin, RuleRef, style::computed_style::ComputedStyle};
pub struct CascadeSource<'a> {
    pub origin: Origin,
    pub matches: Option<Vec<RuleRef>>,
    pub cssom: &'a CssOm,
}
pub struct Cascade;

impl Cascade {
    pub(crate) fn compute(node: &Node, dom: &Dom, sources: &[CascadeSource<'_>]) -> ComputedStyle {
        ComputedStyle {
            properties: HashMap::new(),
        }
    }
}
