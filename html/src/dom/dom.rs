use crate::dom::{node::Node, style_sheet_ref::StyleSheetRef};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dom {
    pub nodes: Vec<Node>,
    pub style_sheets: Vec<StyleSheetRef>,
}

impl Dom {
    pub fn new(nodes: Vec<Node>, style_sheets: Vec<StyleSheetRef>) -> Self {
        Self {
            nodes,
            style_sheets,
        }
    }
}
