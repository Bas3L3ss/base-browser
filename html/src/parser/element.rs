use crate::tokenizer::{attributes::Attributes, html_tags::HtmlTag};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DomElement {
    pub tag: HtmlTag,
    pub attributes: Attributes,
}

impl DomElement {
    pub fn new(tag: HtmlTag, attributes: Attributes) -> Self {
        DomElement { tag, attributes }
    }
}
