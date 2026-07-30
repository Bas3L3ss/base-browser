use std::unimplemented;

use html::dom::{dom::Dom, node::Node};

pub struct Matcher;

impl Matcher {
    pub(crate) fn match_rules(node: &Node, dom: &Dom, candidates: &[u32]) -> Vec<u32> {
        unimplemented!()
    }
}
