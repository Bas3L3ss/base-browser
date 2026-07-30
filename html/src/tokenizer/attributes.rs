use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Attributes {
    pub id: Option<String>,
    pub class: Option<String>,
    pub style: Option<String>,
    pub title: Option<String>,
    pub name: Option<String>,
    pub value: Option<String>,
    pub href: Option<String>,
    pub src: Option<String>,
    pub alt: Option<String>,
    pub type_attr: Option<String>,
    pub placeholder: Option<String>,
    pub tabindex: Option<String>,
    pub disabled: Option<bool>,
    pub checked: Option<bool>,
    pub readonly: Option<bool>,
    pub selected: Option<bool>,
    pub required: Option<bool>,
    pub maxlength: Option<String>,
    pub minlength: Option<String>,
    pub width: Option<String>,
    pub height: Option<String>,
    pub rel: Option<String>,
    pub action: Option<String>,
    pub method: Option<String>,
    pub enctype: Option<String>,
    pub for_attr: Option<String>,
    pub target: Option<String>,
    pub pattern: Option<String>,
    pub autocomplete: Option<String>,
    pub multiple: Option<bool>,
    pub hidden: Option<bool>,
    pub role: Option<String>,
    pub aria_label: Option<String>,
    pub custom: Option<HashMap<String, String>>,
}

impl Attributes {
    pub fn new() -> Self {
        Attributes::default()
    }

    /// Adds or updates the attribute given a key and value.
    pub fn add_attribute(&mut self, key: &str, value: &str) {
        match key {
            "id" => self.id = Some(value.to_string()),
            "class" => self.class = Some(value.to_string()),
            "style" => self.style = Some(value.to_string()),
            "title" => self.title = Some(value.to_string()),
            "name" => self.name = Some(value.to_string()),
            "value" => self.value = Some(value.to_string()),
            "href" => self.href = Some(value.to_string()),
            "src" => self.src = Some(value.to_string()),
            "alt" => self.alt = Some(value.to_string()),
            "type" => self.type_attr = Some(value.to_string()),
            "placeholder" => self.placeholder = Some(value.to_string()),
            "tabindex" => self.tabindex = Some(value.to_string()),
            "disabled" => {
                self.disabled = match value.to_ascii_lowercase().as_str() {
                    "false" | "0" => Some(false),
                    _ => Some(true),
                }
            }
            "checked" => self.checked = Some(true),
            "readonly" => self.readonly = Some(true),
            "selected" => self.selected = Some(true),
            "required" => self.required = Some(true),
            "maxlength" => self.maxlength = Some(value.to_string()),
            "minlength" => self.minlength = Some(value.to_string()),
            "width" => self.width = Some(value.to_string()),
            "height" => self.height = Some(value.to_string()),
            "rel" => self.rel = Some(value.to_string()),
            "action" => self.action = Some(value.to_string()),
            "method" => self.method = Some(value.to_string()),
            "enctype" => self.enctype = Some(value.to_string()),
            "for" => self.for_attr = Some(value.to_string()),
            "target" => self.target = Some(value.to_string()),
            "pattern" => self.pattern = Some(value.to_string()),
            "autocomplete" => self.autocomplete = Some(value.to_string()),
            "multiple" => self.multiple = Some(true),
            "hidden" => self.hidden = Some(true),
            "role" => self.role = Some(value.to_string()),
            "aria-label" => self.aria_label = Some(value.to_string()),
            _ => {
                let mut map = self.custom.take().unwrap_or_default();
                map.insert(key.to_string(), value.to_string());
                self.custom = Some(map);
            }
        }
    }

    /// Returns Some(attribute_value) if present for the given key, or if the attribute is a boolean
    /// and true, returns Some(attribute_key) as a string, otherwise returns None.
    pub fn contains(&self, key: &str) -> Option<String> {
        match key {
            "id" => self.id.clone(),
            "class" => self.class.clone(),
            "style" => self.style.clone(),
            "title" => self.title.clone(),
            "name" => self.name.clone(),
            "value" => self.value.clone(),
            "href" => self.href.clone(),
            "src" => self.src.clone(),
            "alt" => self.alt.clone(),
            "type" => self.type_attr.clone(),
            "placeholder" => self.placeholder.clone(),
            "tabindex" => self.tabindex.clone(),
            "disabled" => self.disabled.and_then(|b| {
                if b {
                    Some("disabled".to_string())
                } else {
                    None
                }
            }),
            "checked" => self
                .checked
                .and_then(|b| if b { Some("checked".to_string()) } else { None }),
            "readonly" => self.readonly.and_then(|b| {
                if b {
                    Some("readonly".to_string())
                } else {
                    None
                }
            }),
            "selected" => self.selected.and_then(|b| {
                if b {
                    Some("selected".to_string())
                } else {
                    None
                }
            }),
            "required" => self.required.and_then(|b| {
                if b {
                    Some("required".to_string())
                } else {
                    None
                }
            }),
            "maxlength" => self.maxlength.clone(),
            "minlength" => self.minlength.clone(),
            "width" => self.width.clone(),
            "height" => self.height.clone(),
            "rel" => self.rel.clone(),
            "action" => self.action.clone(),
            "method" => self.method.clone(),
            "enctype" => self.enctype.clone(),
            "for" => self.for_attr.clone(),
            "target" => self.target.clone(),
            "pattern" => self.pattern.clone(),
            "autocomplete" => self.autocomplete.clone(),
            "multiple" => self.multiple.and_then(|b| {
                if b {
                    Some("multiple".to_string())
                } else {
                    None
                }
            }),
            "hidden" => self
                .hidden
                .and_then(|b| if b { Some("hidden".to_string()) } else { None }),
            "role" => self.role.clone(),
            "aria-label" => self.aria_label.clone(),
            _ => {
                // custom attributes
                self.custom.as_ref().and_then(|map| map.get(key).cloned())
            }
        }
    }
}
