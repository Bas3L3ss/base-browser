use crate::dom::node::NodeID;
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StyleSheetRef {
    InlineStyle(NodeID),
    StyleSheetLink(NodeID),
}
