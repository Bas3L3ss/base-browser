use crate::dom::{node::Node, style_sheet_ref::StyleSheetRef};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dom {
    pub nodes: Vec<Node>,
    pub style_sheets: Vec<StyleSheetRef>,
    pub document_element: usize,
    pub body_element: usize,
    pub head_element: usize,
}

impl Dom {
    pub fn new(
        nodes: Vec<Node>,
        style_sheets: Vec<StyleSheetRef>,
        document_element: usize,
        body_element: usize,
        head_element: usize,
    ) -> Self {
        Self {
            nodes,
            style_sheets,
            document_element,
            body_element,
            head_element,
        }
    }
}
