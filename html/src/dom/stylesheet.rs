use crate::dom::node::NodeID;
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StyleSheet {
    InlineStyle(NodeID),
    StyleSheetLink(NodeID),
}
