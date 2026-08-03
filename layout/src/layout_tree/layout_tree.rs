use std::unimplemented;

use css::ComputedStyle;
use html::Dom;

use crate::{BoxType, LayoutNode};

pub struct LayoutTree {
    pub root: LayoutNode,
}

impl LayoutTree {
    pub fn build(dom: &Dom, computed_styles: &[Option<ComputedStyle>]) -> Self {
        let html_id = dom.document_element; // returns the <html> node id
        unimplemented!()
    }
}
