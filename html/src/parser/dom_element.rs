use crate::tokenizer::{attributes::Attributes, html_tags::HtmlTag};

#[derive(Clone, PartialEq, Eq)]
pub struct DomElement {
    pub tag: HtmlTag,
    pub attributes: Attributes,
}

// Custom Debug implementation that omits attributes
impl std::fmt::Debug for DomElement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DomElement")
            .field("tag", &self.tag)
            .finish()
    }
}

impl DomElement {
    pub fn new(tag: HtmlTag, attributes: Attributes) -> Self {
        DomElement { tag, attributes }
    }
}
