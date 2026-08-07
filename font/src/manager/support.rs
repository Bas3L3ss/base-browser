// support.rs

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SupportedFormat {
    TrueType,
    OpenType,
    Woff,
    Woff2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FontFileFormat {
    TrueType,
    OpenType,
    Woff,
    Woff2,
    Eot,
    Svg,
    Collection,
    Bitmap,
    Unknown,
}

pub const SUPPORTED_FORMATS: &[SupportedFormat] = &[
    SupportedFormat::TrueType,
    SupportedFormat::OpenType,
    SupportedFormat::Woff,
    SupportedFormat::Woff2,
];

pub(crate) struct FontSupport;

impl FontSupport {
    pub(crate) fn supports_tech(_techs: &[css::FontTechnology]) -> bool {
        true
    }

    /// Detect the font file format from file bytes.
    pub fn detect_format_from_bytes(bytes: &[u8]) -> FontFileFormat {
        if bytes.len() < 4 {
            return FontFileFormat::Unknown;
        }

        match &bytes[..4] {
            [0x00, 0x01, 0x00, 0x00] => FontFileFormat::TrueType,
            b"OTTO" => FontFileFormat::OpenType,
            b"wOFF" => FontFileFormat::Woff,
            b"wOF2" => FontFileFormat::Woff2,
            // EOT format has: 0x4C 0x50 0x00 0x00 (b"L P\0\0") at <start>
            [0x4C, 0x50, 0x00, 0x00] => FontFileFormat::Eot,
            // SVG: look for <svg ...> start tag. Check first 5 (could be whitespace padded)
            _ => {
                // Try SVG (first few non-whitespace chars as "<svg")
                if bytes.len() >= 5 {
                    let mut idx = 0;
                    while idx < 5 && bytes[idx].is_ascii_whitespace() {
                        idx += 1;
                    }
                    if bytes.get(idx..idx + 4) == Some(b"<svg") {
                        return FontFileFormat::Svg;
                    }
                }
                // TTC: font collection: b"ttcf"
                if bytes.len() >= 4 && &bytes[..4] == b"ttcf" {
                    return FontFileFormat::Collection;
                }
                // Bitmap fonts: BDF (start 'STARTFONT'), PCF (first 4: 1f 43 01 00, etc), FNT (not easily detected here)
                if bytes.len() >= 9 && &bytes[..9] == b"STARTFONT" {
                    return FontFileFormat::Bitmap;
                }
                // PCF detection (first byte 0x01, then 0x66, 0x43, 0x01)
                if bytes.len() >= 4
                    && bytes[0] == 0x01
                    && bytes[1] == 0x66
                    && bytes[2] == 0x43
                    && bytes[3] == 0x01
                {
                    return FontFileFormat::Bitmap;
                }
                FontFileFormat::Unknown
            }
        }
    }

    /// Detect the font file format from the given CSS format list and/or url.
    pub(crate) fn detect_format_from_css(
        formats: &[css::FontFormat],
        href: &str,
    ) -> FontFileFormat {
        // Prioritize explicit CSS format first
        if !formats.is_empty() {
            // Use the first supported found, else Unknown
            for format in formats {
                match format {
                    css::FontFormat::TrueType => return FontFileFormat::TrueType,
                    css::FontFormat::OpenType => return FontFileFormat::OpenType,
                    css::FontFormat::Woff => return FontFileFormat::Woff,
                    css::FontFormat::Woff2 => return FontFileFormat::Woff2,
                    css::FontFormat::EmbeddedOpenType => return FontFileFormat::Eot,
                    css::FontFormat::Svg => return FontFileFormat::Svg,
                    css::FontFormat::Other(s) if s.eq_ignore_ascii_case("ttc") => {
                        return FontFileFormat::Collection;
                    }
                    css::FontFormat::Other(s)
                        if s.eq_ignore_ascii_case("bdf")
                            || s.eq_ignore_ascii_case("pcf")
                            || s.eq_ignore_ascii_case("fnt") =>
                    {
                        return FontFileFormat::Bitmap;
                    }

                    _ => {}
                }
            }
            return FontFileFormat::Unknown;
        }

        // Infer from file extension
        let ext = href
            .rsplit('.')
            .next()
            .map(|s| s.trim().to_ascii_lowercase());
        match ext.as_deref() {
            Some("ttf") => FontFileFormat::TrueType,
            Some("otf") => FontFileFormat::OpenType,
            Some("woff") => FontFileFormat::Woff,
            Some("woff2") => FontFileFormat::Woff2,
            Some("eot") => FontFileFormat::Eot,
            Some("svg") => FontFileFormat::Svg,
            Some("ttc") => FontFileFormat::Collection,
            Some("bdf") | Some("pcf") | Some("fnt") => FontFileFormat::Bitmap,
            Some(_) | None => FontFileFormat::Unknown,
        }
    }

    /// Returns true if a FontFileFormat is supported by the current set.
    /// NOTE: Unknown is always "supported" per instruction.
    pub(crate) fn supports_format(format: FontFileFormat) -> bool {
        match format {
            FontFileFormat::TrueType => SUPPORTED_FORMATS.contains(&SupportedFormat::TrueType),
            FontFileFormat::OpenType => SUPPORTED_FORMATS.contains(&SupportedFormat::OpenType),
            FontFileFormat::Woff => SUPPORTED_FORMATS.contains(&SupportedFormat::Woff),
            FontFileFormat::Woff2 => SUPPORTED_FORMATS.contains(&SupportedFormat::Woff2),
            // Extended/unsupported formats
            FontFileFormat::Eot
            | FontFileFormat::Svg
            | FontFileFormat::Collection
            | FontFileFormat::Bitmap => false,
            FontFileFormat::Unknown => true,
        }
    }
}
