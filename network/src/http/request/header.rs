#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpHeader {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpHeaders {
    headers: Vec<HttpHeader>,
}

impl HttpHeaders {
    pub fn new() -> Self {
        Self {
            headers: Vec::new(),
        }
    }

    /// Insert or replace a header (by case-insensitive name)
    pub fn set(&mut self, name: &str, value: &str) {
        if let Some(header) = self
            .headers
            .iter_mut()
            .find(|h| h.name.eq_ignore_ascii_case(name))
        {
            header.value = value.to_string();
        } else {
            self.headers.push(HttpHeader {
                name: name.to_string(),
                value: value.to_string(),
            });
        }
    }

    /// Remove a header (by case-insensitive name)
    pub fn remove(&mut self, name: &str) {
        self.headers.retain(|h| !h.name.eq_ignore_ascii_case(name));
    }

    /// Get a reference to the value of the header, if present (case-insensitive)
    pub fn get(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|h| h.name.eq_ignore_ascii_case(name))
            .map(|h| h.value.as_str())
    }

    /// Iterate header key-value pairs for serialization
    pub fn iter(&self) -> impl Iterator<Item = &HttpHeader> {
        self.headers.iter()
    }

    /// Check if there are no headers
    pub fn is_empty(&self) -> bool {
        self.headers.is_empty()
    }
}
