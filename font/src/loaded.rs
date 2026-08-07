use std::sync::Arc;

use crate::FontError;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct LoadedFont {
    data: Arc<[u8]>,
}

impl LoadedFont {
    pub fn face(&self) -> Result<ttf_parser::Face<'_>, FontError> {
        ttf_parser::Face::parse(&self.data, 0).map_err(|e| FontError::ParseError(e.to_string()))
    }

    pub fn from_bytes(data: Vec<u8>) -> Self {
        let data: Arc<[u8]> = data.into();
        Self { data }
    }
}
