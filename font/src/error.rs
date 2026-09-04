use std::write;

pub enum FontError {
    ParseError(String),
    GlyphNotFound(String),
}

impl std::fmt::Debug for FontError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FontError::ParseError(s) => {
                write!(f, "Font parsing error: {}", s)
            }
            FontError::GlyphNotFound(s) => {
                write!(f, "Glyph not found: '{}'", s)
            }
        }
    }
}
