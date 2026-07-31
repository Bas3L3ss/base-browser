use crate::tokenizer::{attributes::Attributes, html_tags::HtmlTag};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct DomElement {
    pub tag: HtmlTag,
    pub attributes: Attributes,
}

impl DomElement {
    pub fn new(tag: HtmlTag, attributes: Attributes) -> Self {
        DomElement { tag, attributes }
    }

    /// Returns a string representation of the tag name.
    pub fn tag_name(&self) -> String {
        self.tag.to_tag_string()
    }

    /// Returns the value of the "id" attribute if present.
    pub fn id(&self) -> Option<&str> {
        self.attributes.id.as_deref()
    }

    /// Returns a vector of class names in the "class" attribute, if any.
    pub fn classes(&self) -> Vec<&str> {
        match &self.attributes.class {
            Some(class_str) => class_str.split_whitespace().collect(),
            None => Vec::new(),
        }
    }
}
