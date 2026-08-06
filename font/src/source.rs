use std::{fmt, matches, write};

use url::Url;

/// Where a font can be loaded from.
///
/// This is the runtime representation of a font source.
/// CSS parsing creates css::FontSource, then the font system
/// converts it into this type.
#[derive(Debug, Clone)]
pub enum FontSource {
    /// A downloadable font resource.
    Url(FontUrl),

    /// A font already installed on the system.
    Local(String),
}

#[derive(Debug, Clone)]
pub struct FontUrl {
    pub url: Url,

    pub format: Option<FontFormat>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontFormat {
    Woff,
    Woff2,
    TrueType,
    OpenType,
    EmbeddedOpenType,
    Svg,
}

impl FontSource {
    pub fn is_remote(&self) -> bool {
        matches!(self, FontSource::Url(_))
    }

    pub fn is_local(&self) -> bool {
        matches!(self, FontSource::Local(_))
    }
}

impl fmt::Display for FontSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FontSource::Url(font_url) => {
                write!(f, "url({})", font_url.url)
            }

            FontSource::Local(name) => {
                write!(f, "local({})", name)
            }
        }
    }
}
