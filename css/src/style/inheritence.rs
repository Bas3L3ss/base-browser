use std::unimplemented;

use html::dom::{dom::Dom, node::Node};

use crate::style::computed_style::ComputedStyle;

pub struct Inheritance;

impl Inheritance {
    pub(crate) fn apply(node: &Node, dom: &Dom, style: &mut ComputedStyle) {
        unimplemented!()
    }
}
