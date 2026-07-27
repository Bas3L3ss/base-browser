use crate::dom::dom_element::DomElement;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeType {
    Document,
    Doctype(String),
    Element(DomElement),
    Text(String),
    Comment(String),
}
