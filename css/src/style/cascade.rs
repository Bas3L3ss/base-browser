use std::unimplemented;

use html::dom::{dom::Dom, node::Node};

use crate::style::computed_style::ComputedStyle;

pub struct Cascade;

impl Cascade {
    pub(crate) fn compute(node: &Node, dom: &Dom, matching: Option<&[u32]>) -> ComputedStyle {
        unimplemented!()
    }
}
