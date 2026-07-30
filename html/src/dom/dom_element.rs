use std::format;

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
        format!("{:?}", self.tag)
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

    /// Returns a vector of all attribute names present on the element (including standard and custom attributes).
    /// Only returns the key string, e.g. `data="idk"` will yield ["data"].
    pub fn attribute_keys(&self) -> Vec<&str> {
        let mut keys = Vec::new();

        // Known attribute fields
        if self.attributes.id.is_some() {
            keys.push("id");
        }
        if self.attributes.class.is_some() {
            keys.push("class");
        }
        if self.attributes.style.is_some() {
            keys.push("style");
        }
        if self.attributes.title.is_some() {
            keys.push("title");
        }
        if self.attributes.name.is_some() {
            keys.push("name");
        }
        if self.attributes.value.is_some() {
            keys.push("value");
        }
        if self.attributes.href.is_some() {
            keys.push("href");
        }
        if self.attributes.src.is_some() {
            keys.push("src");
        }
        if self.attributes.alt.is_some() {
            keys.push("alt");
        }
        if self.attributes.type_attr.is_some() {
            keys.push("type");
        }
        if self.attributes.placeholder.is_some() {
            keys.push("placeholder");
        }
        if self.attributes.tabindex.is_some() {
            keys.push("tabindex");
        }
        if let Some(disabled) = self.attributes.disabled {
            if disabled {
                keys.push("disabled");
            }
        }
        if let Some(checked) = self.attributes.checked {
            if checked {
                keys.push("checked");
            }
        }
        if let Some(readonly) = self.attributes.readonly {
            if readonly {
                keys.push("readonly");
            }
        }
        if let Some(selected) = self.attributes.selected {
            if selected {
                keys.push("selected");
            }
        }
        if let Some(required) = self.attributes.required {
            if required {
                keys.push("required");
            }
        }
        if self.attributes.maxlength.is_some() {
            keys.push("maxlength");
        }
        if self.attributes.minlength.is_some() {
            keys.push("minlength");
        }
        if self.attributes.width.is_some() {
            keys.push("width");
        }
        if self.attributes.height.is_some() {
            keys.push("height");
        }
        if self.attributes.rel.is_some() {
            keys.push("rel");
        }
        if self.attributes.action.is_some() {
            keys.push("action");
        }
        if self.attributes.method.is_some() {
            keys.push("method");
        }
        if self.attributes.enctype.is_some() {
            keys.push("enctype");
        }
        if self.attributes.for_attr.is_some() {
            keys.push("for");
        }
        if self.attributes.target.is_some() {
            keys.push("target");
        }
        if self.attributes.pattern.is_some() {
            keys.push("pattern");
        }
        if self.attributes.autocomplete.is_some() {
            keys.push("autocomplete");
        }
        if let Some(multiple) = self.attributes.multiple {
            if multiple {
                keys.push("multiple");
            }
        }
        if let Some(hidden) = self.attributes.hidden {
            if hidden {
                keys.push("hidden");
            }
        }
        if self.attributes.role.is_some() {
            keys.push("role");
        }
        if self.attributes.aria_label.is_some() {
            keys.push("aria-label");
        }

        // Custom attributes (e.g. data-* or other arbitrary attributes)
        if let Some(custom) = &self.attributes.custom {
            for key in custom.keys() {
                keys.push(key.as_str());
            }
        }

        keys
    }
}
