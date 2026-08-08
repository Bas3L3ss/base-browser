use std::{matches, unreachable, vec};

use crate::{Angle, AngleUnit, AtRule, AtRuleBlock, ComponentValue, Property};

#[derive(Debug, Clone, PartialEq)]
pub enum FontStyle {
    Normal,
    Italic,
    /// For Oblique, support both single angle and range.
    Oblique(ObliqueStyle),
}

impl Default for FontStyle {
    fn default() -> Self {
        FontStyle::Normal
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ObliqueStyle {
    /// oblique <angle>
    Single(Angle),
    /// oblique <angle1> <angle2>
    Range(Angle, Angle),
}

impl Default for ObliqueStyle {
    fn default() -> Self {
        // Default to oblique 14deg per CSS spec
        ObliqueStyle::Single(Angle {
            value: 14.0,
            unit: AngleUnit::Deg,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FontWeight {
    Single(u16),
    Range { min: u16, max: u16 },
}

impl Default for FontWeight {
    fn default() -> Self {
        FontWeight::Single(400)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FontStretch {
    UltraCondensed,
    ExtraCondensed,
    Condensed,
    SemiCondensed,
    Normal,
    SemiExpanded,
    Expanded,
    ExtraExpanded,
    UltraExpanded,
    Range { min: f32, max: f32 },
}

impl Default for FontStretch {
    fn default() -> Self {
        FontStretch::Normal
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FontDisplay {
    Auto,
    Block,
    Swap,
    Fallback,
    Optional,
}

impl Default for FontDisplay {
    fn default() -> Self {
        FontDisplay::Auto
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FontTechnology {
    Variations,
    Palettes,
    Features,
    Color,
    Other(String),
}

impl Default for FontTechnology {
    fn default() -> Self {
        FontTechnology::Variations
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FontSource {
    Url {
        url: String,
        formats: Vec<FontFormat>,
        techs: Vec<FontTechnology>,
    },
    Local {
        family: String,
    },
}

impl Default for FontSource {
    fn default() -> Self {
        FontSource::Url {
            url: String::new(),
            formats: vec![],
            techs: vec![],
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FontFormat {
    Woff,
    Woff2,
    TrueType,
    OpenType,
    EmbeddedOpenType,
    Svg,
    /// Unknown/custom formats
    Other(String),
}

impl Default for FontFormat {
    fn default() -> Self {
        FontFormat::Woff
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnicodeRange {
    pub start: u32,
    pub end: u32,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct FontFaceRule {
    pub family: String,
    pub sources: Vec<FontSource>,
    pub style: FontStyle,
    pub weight: FontWeight,
    pub stretch: FontStretch,
    pub display: FontDisplay,
    pub unicode_range: Vec<UnicodeRange>,
}

impl FontFaceRule {
    pub fn from_rule(rule: &AtRule) -> Self {
        if let Some(block) = &rule.block {
            return match block {
                AtRuleBlock::DeclarationList(list) => {
                    let mut font_face = FontFaceRule::default();
                    for declaration in list {
                        match &declaration.property {
                            Property::Src => {
                                let mut sources: Vec<FontSource> = Vec::new();
                                let mut iter = declaration.value.components.iter().peekable();
                                while let Some(value) = iter.next() {
                                    match value {
                                        ComponentValue::Function(func) => {
                                            match func.name.as_str() {
                                                "local" => {
                                                    if let Some(family) =
                                                        Self::parse_local_family(&func.args)
                                                    {
                                                        sources.push(FontSource::Local { family });
                                                    }
                                                }
                                                "url" => {
                                                    if let Some(url) = Self::parse_url(&func.args) {
                                                        let mut techs = Vec::new();
                                                        let mut formats = Vec::new();

                                                        // Use a while loop to consume all tech or format functions after url
                                                        while let Some(ComponentValue::Function(
                                                            func,
                                                        )) = iter.peek()
                                                        {
                                                            match func.name.as_str() {
                                                                "tech" => {
                                                                    Self::parse_techs(
                                                                        &func.args, &mut techs,
                                                                    );
                                                                    iter.next(); // consume "tech"
                                                                }
                                                                "format" => {
                                                                    Self::parse_formats(
                                                                        &func.args,
                                                                        &mut formats,
                                                                    );
                                                                    iter.next(); // consume "format"
                                                                }
                                                                _ => break,
                                                            }
                                                        }

                                                        sources.push(FontSource::Url {
                                                            url,
                                                            formats,
                                                            techs,
                                                        });
                                                    }
                                                }

                                                _ => {}
                                            }
                                        }
                                        _ => {}
                                    };
                                }
                                font_face.sources = sources;
                            }
                            Property::FontFamily => {
                                if let Some(first) = declaration.value.components.first() {
                                    match first {
                                        ComponentValue::String(s) => {
                                            font_face.family = s.clone();
                                        }
                                        ComponentValue::Ident(_) => {
                                            let mut family_parts = vec![];
                                            let mut only_idents = true;
                                            for arg in &declaration.value.components {
                                                match arg {
                                                    ComponentValue::Ident(s) => {
                                                        family_parts.push(s.as_str());
                                                    }
                                                    _ => {
                                                        only_idents = false;
                                                        break;
                                                    }
                                                }
                                            }
                                            if only_idents && !family_parts.is_empty() {
                                                font_face.family = family_parts.join(" ");
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                            }

                            Property::FontStyle => {
                                // Set style from value (normalize to FontStyle)
                                if let Some(style_val) = declaration.value.components.first() {
                                    // Match known font-style strings
                                    font_face.style = match style_val.as_ident() {
                                        Some("normal") => FontStyle::Normal,
                                        Some("italic") => FontStyle::Italic,
                                        Some("oblique") => {
                                            Self::parse_oblique(&declaration.value.components[1..])
                                        }
                                        _ => FontStyle::Normal,
                                    };
                                }
                            }
                            Property::FontWeight => {
                                let comps = &declaration.value.components;
                                font_face.weight = {
                                    if let Some(first) = comps.get(0) {
                                        let first_weight = Self::parse_weight(first);
                                        match comps.get(1) {
                                            // Range: two weights
                                            Some(second) => {
                                                let second_weight = Self::parse_weight(second);
                                                match (first_weight, second_weight) {
                                                    (Some(min), Some(max)) => {
                                                        FontWeight::Range { min, max }
                                                    }
                                                    _ => FontWeight::default(),
                                                }
                                            }
                                            // Single: just one value
                                            None => first_weight
                                                .map(FontWeight::Single)
                                                .unwrap_or_else(FontWeight::default),
                                        }
                                    } else {
                                        // If no components, weight is set to default
                                        FontWeight::default()
                                    }
                                };
                            }

                            Property::FontStretch => {
                                let comps = &declaration.value.components;
                                let stretch = match (comps.get(0), comps.get(1)) {
                                    (Some(first), Some(second)) => {
                                        let first_val = match first {
                                            ComponentValue::Percentage(pct) => Some(*pct),
                                            ComponentValue::Number(num) => Some(*num),
                                            _ => None,
                                        };
                                        let second_val = match second {
                                            ComponentValue::Percentage(pct) => Some(*pct),
                                            ComponentValue::Number(num) => Some(*num),
                                            _ => None,
                                        };
                                        match (first_val, second_val) {
                                            (Some(min), Some(max)) => {
                                                Some(FontStretch::Range { min, max })
                                            }
                                            _ => None,
                                        }
                                    }
                                    (Some(first), None) => match first.as_ident() {
                                        Some("ultra-condensed") => {
                                            Some(FontStretch::UltraCondensed)
                                        }
                                        Some("extra-condensed") => {
                                            Some(FontStretch::ExtraCondensed)
                                        }
                                        Some("condensed") => Some(FontStretch::Condensed),
                                        Some("semi-condensed") => Some(FontStretch::SemiCondensed),
                                        Some("normal") => Some(FontStretch::Normal),
                                        Some("semi-expanded") => Some(FontStretch::SemiExpanded),
                                        Some("expanded") => Some(FontStretch::Expanded),
                                        Some("extra-expanded") => Some(FontStretch::ExtraExpanded),
                                        Some("ultra-expanded") => Some(FontStretch::UltraExpanded),
                                        _ => None,
                                    },
                                    _ => None,
                                };
                                if let Some(valid_stretch) = stretch {
                                    font_face.stretch = valid_stretch;
                                }
                            }

                            Property::Unknown(str) if str.as_str() == "unicode-range" => {
                                // Because of tokenizer implementation, a "unicode-range" may be
                                // We need to buffer all tokens up to a comma, turn them into a String via to_string(),
                                // then parse as a unicode range

                                let mut ranges = Vec::new();
                                let mut failed = false;

                                let comps = &declaration.value.components;
                                let mut buffer = String::new();
                                let mut i = 0;

                                while i <= comps.len() {
                                    // Also handle the terminator (last segment)
                                    if i == comps.len()
                                        || matches!(comps.get(i), Some(ComponentValue::Delim(',')))
                                    {
                                        if !buffer.trim().is_empty() {
                                            if let Some(range) =
                                                Self::parse_unicode_range(buffer.trim())
                                            {
                                                ranges.push(range);
                                            } else {
                                                failed = true;
                                                break;
                                            }
                                            buffer.clear();
                                        }
                                        i += 1; // skip comma or end
                                        continue;
                                    }
                                    // Otherwise buffer this component's string representation
                                    if let Some(comp) = comps.get(i) {
                                        buffer.push_str(&comp.to_string());
                                    }
                                    i += 1;
                                }

                                if !ranges.is_empty() && !failed {
                                    font_face.unicode_range = ranges;
                                }
                            }

                            Property::Unknown(str) if str.as_str() == "font-display" => {
                                if let Some(display_val) = declaration.value.components.first() {
                                    font_face.display = match display_val.as_ident() {
                                        Some("auto") => FontDisplay::Auto,
                                        Some("block") => FontDisplay::Block,
                                        Some("swap") => FontDisplay::Swap,
                                        Some("fallback") => FontDisplay::Fallback,
                                        Some("optional") => FontDisplay::Optional,
                                        _ => FontDisplay::Auto,
                                    };
                                }
                            }
                            _ => {}
                        }
                    }
                    font_face
                }
                _ => Self::default(),
            };
        }
        Self::default()
    }

    fn parse_unicode_range(s: &str) -> Option<UnicodeRange> {
        if let Some(rest) = s.strip_prefix("U+") {
            // Handle U+XXXX-YYYY
            if let Some((from, to)) = rest.split_once('-') {
                let from_parsed = Self::parse_unicode_hex_or_wildcard(from)?;
                let to_parsed = Self::parse_unicode_hex_or_wildcard(to)?;
                Some(UnicodeRange {
                    start: from_parsed.0,
                    end: to_parsed.1,
                })
            }
            // Handle U+XX??
            else if rest.contains('?') {
                let (start, end) = Self::parse_unicode_hex_or_wildcard(rest)?;
                Some(UnicodeRange { start, end })
            }
            // Handle U+XXXX (single)
            else {
                let cp = u32::from_str_radix(rest, 16).ok()?;
                Some(UnicodeRange { start: cp, end: cp })
            }
        } else {
            None
        }
    }
    // Parse a string like "1234", "12??", "0041"
    fn parse_unicode_hex_or_wildcard(s: &str) -> Option<(u32, u32)> {
        let up = s.to_ascii_uppercase();
        // Handle wildcards: replace ? with 0 for start, F for end
        let wildcard_count = up.chars().filter(|&c| c == '?').count();
        if wildcard_count > 0 && up.chars().all(|c| c.is_ascii_hexdigit() || c == '?') {
            let base: String = up.chars().map(|c| if c == '?' { '0' } else { c }).collect();
            let start = u32::from_str_radix(&base, 16).ok()?;
            let base: String = up.chars().map(|c| if c == '?' { 'F' } else { c }).collect();
            let end = u32::from_str_radix(&base, 16).ok()?;
            Some((start, end))
        } else if up.chars().all(|c| c.is_ascii_hexdigit()) {
            // No wildcards, it's a simple number
            let v = u32::from_str_radix(&up, 16).ok()?;
            Some((v, v))
        } else {
            None
        }
    }

    fn parse_angle(val: &ComponentValue) -> Option<Angle> {
        match val {
            ComponentValue::Dimension { value, unit } => match unit.to_string().as_str() {
                "deg" => Some(Angle {
                    value: *value,
                    unit: AngleUnit::Deg,
                }),
                "grad" => Some(Angle {
                    value: *value,
                    unit: AngleUnit::Grad,
                }),
                "rad" => Some(Angle {
                    value: *value,
                    unit: AngleUnit::Rad,
                }),
                "turn" => Some(Angle {
                    value: *value,
                    unit: AngleUnit::Turn,
                }),
                _ => None,
            },
            _ => None,
        }
    }

    fn parse_oblique(vals: &[ComponentValue]) -> FontStyle {
        let mut vals = vals.iter();

        let first = vals.next();
        match first {
            // No angle provided, use default 14deg for both min & max
            None => FontStyle::Oblique(ObliqueStyle::Single(Angle {
                value: 14.0,
                unit: AngleUnit::Deg,
            })),
            Some(first_val) => {
                let first_angle = Self::parse_angle(first_val);
                let second = vals.next();
                match second {
                    // oblique <angle1> <angle2>: min=angle1, max=angle2
                    Some(second_val) => {
                        let second_angle = Self::parse_angle(second_val);
                        match (first_angle, second_angle) {
                            (Some(a1), Some(a2)) => FontStyle::Oblique(ObliqueStyle::Range(a1, a2)),
                            _ => FontStyle::Oblique(ObliqueStyle::Single(Angle {
                                value: 14.0,
                                unit: AngleUnit::Deg,
                            })),
                        }
                    }
                    // oblique <angle>: min=max=angle
                    None => first_angle
                        .map(|a| FontStyle::Oblique(ObliqueStyle::Single(a)))
                        .unwrap_or(FontStyle::Oblique(ObliqueStyle::Single(Angle {
                            value: 14.0,
                            unit: AngleUnit::Deg,
                        }))),
                }
            }
        }
    }

    fn parse_weight(val: &ComponentValue) -> Option<u16> {
        match val {
            ComponentValue::Number(n) => Some(*n as u16),
            ComponentValue::Ident(ident) => match ident.as_str() {
                "normal" => Some(400),
                "bold" => Some(700),
                _ => ident.parse::<u16>().ok(),
            },
            _ => None,
        }
    }

    fn parse_url(args: &[ComponentValue]) -> Option<String> {
        if let Some(ComponentValue::String(s)) = args.first() {
            return Some(s.clone());
        }

        let url: String = args.iter().map(ToString::to_string).collect();

        if url.is_empty() { None } else { Some(url) }
    }

    fn parse_techs(args: &[ComponentValue], techs: &mut Vec<FontTechnology>) {
        let mut iter = args.iter().peekable();
        while let Some(arg) = iter.next() {
            match arg {
                ComponentValue::String(s) | ComponentValue::Ident(s) => {
                    let tech = match s.to_ascii_lowercase().as_str() {
                        "variations" => FontTechnology::Variations,
                        "palettes" => FontTechnology::Palettes,
                        "features" => FontTechnology::Features,
                        "color" => FontTechnology::Color,
                        other => FontTechnology::Other(other.to_owned()),
                    };
                    techs.push(tech);
                }
                ComponentValue::Delim(',') => {
                    // skip, comma is just separator
                }
                _ => {
                    // If there's anything else, stop parsing (fail)
                    return;
                }
            }
        }
    }

    fn parse_formats(args: &[ComponentValue], formats: &mut Vec<FontFormat>) {
        // Only accept if all tokens are String or Ident; otherwise, fail (do not push anything)
        if args
            .iter()
            .any(|arg| !matches!(arg, ComponentValue::String(_) | ComponentValue::Ident(_)))
        {
            return; // Fail if any non-ident/string component found
        }
        for arg in args {
            match arg {
                ComponentValue::String(s) | ComponentValue::Ident(s) => {
                    let format = match s.to_ascii_lowercase().as_str() {
                        "woff" => FontFormat::Woff,
                        "woff2" => FontFormat::Woff2,
                        "truetype" | "ttf" => FontFormat::TrueType,
                        "opentype" | "otf" => FontFormat::OpenType,
                        "embedded-opentype" => FontFormat::EmbeddedOpenType,
                        "svg" => FontFormat::Svg,
                        other => FontFormat::Other(other.to_string()),
                    };
                    formats.push(format);
                }
                _ => {
                    unreachable!()
                }
            }
        }
    }

    fn parse_local_family(args: &[ComponentValue]) -> Option<String> {
        // local("Family Name") or local(Family Name [as multiple Idents])
        match args.first() {
            Some(ComponentValue::String(s)) => Some(s.clone()),
            Some(ComponentValue::Ident(_)) => {
                let mut family = String::new();
                for arg in args {
                    match arg {
                        ComponentValue::Ident(s) => {
                            if !family.is_empty() {
                                family.push(' ');
                            }
                            family.push_str(s);
                        }
                        // Stop and invalidate on any non-Ident
                        _ => return None,
                    }
                }
                if family.is_empty() {
                    None
                } else {
                    Some(family)
                }
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use core::panic;
    use std::assert_eq;
    use std::vec;

    use super::*;
    use crate::parser::parser::Parser;
    use crate::rule::at_rule::AtRule;
    use crate::rule::at_rule::AtRuleBlock;
    use crate::rule::rule::CssRule;
    use crate::tokenizer::tokenizer::Tokenizer;

    fn parse_font_face_at_rule(input: &str) -> AtRule {
        let tokens = Tokenizer::from_string(input);
        let mut parser = Parser::new(&tokens);

        let rules = parser.parse().expect("Should parse successfully");

        match &rules[0] {
            CssRule::At(at_rule) => at_rule.clone(),
            other => panic!("Expected AtRule, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_simple_font_face() {
        let input = r#"
            @font-face {
                font-family: "MyFont";
                src: url(myfont.woff2);
            }
        "#;
        let at_rule = parse_font_face_at_rule(input);
        let ff = FontFaceRule::from_rule(&at_rule);
        assert_eq!(ff.family, "MyFont");
        assert_eq!(
            ff.sources,
            vec![FontSource::Url {
                url: "myfont.woff2".to_string(),
                formats: vec![],
                techs: vec![]
            }]
        );
        assert_eq!(ff.style, FontStyle::default());
        assert_eq!(ff.weight, FontWeight::default());
        assert_eq!(ff.stretch, FontStretch::default());
        assert_eq!(ff.display, FontDisplay::default());
        assert_eq!(ff.unicode_range, vec![]);
    }

    #[test]
    fn test_parse_font_face_with_italic_style() {
        let input = r#"
            @font-face {
                font-family: "ItalicFont";
                font-style: italic;
                src: url("italic.woff2");
            }
        "#;
        let at_rule = parse_font_face_at_rule(input);
        let ff = FontFaceRule::from_rule(&at_rule);
        assert_eq!(ff.family, "ItalicFont");
        assert_eq!(ff.style, FontStyle::Italic);
        assert_eq!(
            ff.sources,
            vec![FontSource::Url {
                url: "italic.woff2".to_string(),
                formats: vec![],
                techs: vec![]
            }]
        );
    }

    #[test]
    fn test_parse_font_face_with_oblique_angle() {
        let input = r#"
            @font-face {
                font-family: Example;
                font-style: oblique 20deg;
                src: url(example.ttf);
            }
        "#;
        let at_rule = parse_font_face_at_rule(input);
        let ff = FontFaceRule::from_rule(&at_rule);
        assert_eq!(ff.family, "Example");
        assert_eq!(
            ff.style,
            FontStyle::Oblique(ObliqueStyle::Single(Angle {
                value: 20.0,
                unit: AngleUnit::Deg
            }))
        );
        assert_eq!(
            ff.sources,
            vec![FontSource::Url {
                url: "example.ttf".to_string(),
                formats: vec![],
                techs: vec![]
            }]
        );
    }

    #[test]
    fn test_parse_font_face_with_local_source() {
        let input = r#"
            @font-face {
                font-family: "LocalFont";
                src: local("Arial");
            }
        "#;
        let at_rule = parse_font_face_at_rule(input);
        let ff = FontFaceRule::from_rule(&at_rule);
        assert_eq!(ff.family, "LocalFont");
        assert_eq!(
            ff.sources,
            vec![FontSource::Local {
                family: "Arial".to_string()
            }]
        );
    }

    #[test]
    fn test_parse_font_face_with_unicode_range() {
        let input = r#"
            @font-face {
                font-family: "UnicodeFont";
                src: url("unic.ttf");
                unicode-range: U+0025-00FF, U+4E00-9FFF;
            }
        "#;
        let at_rule = parse_font_face_at_rule(input);
        let ff = FontFaceRule::from_rule(&at_rule);
        assert_eq!(ff.family, "UnicodeFont");
        assert_eq!(
            ff.sources,
            vec![FontSource::Url {
                url: "unic.ttf".to_string(),
                formats: vec![],
                techs: vec![]
            }]
        );
        assert_eq!(
            ff.unicode_range,
            vec![
                UnicodeRange {
                    start: 0x0025,
                    end: 0x00FF
                },
                UnicodeRange {
                    start: 0x4E00,
                    end: 0x9FFF
                }
            ]
        );
    }

    #[test]
    fn test_parse_font_face_with_all_fields() {
        let input = r#"
            @font-face {
                font-family: "AllFields";
                font-style: oblique 10deg;
                font-weight: 100 900;
                font-stretch: expanded;
                font-display: swap;
                src: url("a.woff2") format("woff2"), local("A Font");
                unicode-range: U+00-FF, U+1234;
            }
        "#;
        let at_rule = parse_font_face_at_rule(input);
        let ff = FontFaceRule::from_rule(&at_rule);
        assert_eq!(ff.family, "AllFields");
        assert_eq!(
            ff.style,
            FontStyle::Oblique(ObliqueStyle::Single(Angle {
                value: 10.0,
                unit: AngleUnit::Deg
            }))
        );
        assert_eq!(ff.weight, FontWeight::Range { min: 100, max: 900 });
        assert_eq!(ff.stretch, FontStretch::Expanded);
        assert_eq!(ff.display, FontDisplay::Swap);
        assert_eq!(
            ff.sources,
            vec![
                FontSource::Url {
                    url: "a.woff2".to_string(),
                    formats: vec![FontFormat::Woff2],

                    techs: vec![]
                },
                FontSource::Local {
                    family: "A Font".to_string()
                }
            ]
        );
        assert_eq!(
            ff.unicode_range,
            vec![
                UnicodeRange {
                    start: 0x00,
                    end: 0xFF
                },
                UnicodeRange {
                    start: 0x1234,
                    end: 0x1234
                },
            ]
        );
    }

    #[test]
    fn test_parse_font_face_with_minimal_fields() {
        let input = r#"
            @font-face {
                font-family: SimpleFamily;
                src: url(some.ttf);
            }
        "#;
        let at_rule = parse_font_face_at_rule(input);
        let ff = FontFaceRule::from_rule(&at_rule);

        assert_eq!(ff.family, "SimpleFamily");
        assert_eq!(
            ff.sources,
            vec![FontSource::Url {
                url: "some.ttf".to_string(),
                formats: vec![],
                techs: vec![],
            }]
        );
        assert_eq!(ff.style, FontStyle::default());
        assert_eq!(ff.weight, FontWeight::default());
        assert_eq!(ff.stretch, FontStretch::default());
        assert_eq!(ff.display, FontDisplay::default());
        assert_eq!(ff.unicode_range, vec![]);
    }

    #[test]
    fn test_parse_font_face_with_multiple_formats_and_techs() {
        let input = r#"
            @font-face {
                font-family: MultiSource;
                src: url("multi.woff2") format("woff2") tech("variations", "palettes"), 
                     local('FooBar');
            }
        "#;
        let at_rule = parse_font_face_at_rule(input);
        let ff = FontFaceRule::from_rule(&at_rule);
        println!("{ff:#?}");

        assert_eq!(ff.family, "MultiSource");
        assert_eq!(
            ff.sources,
            vec![
                FontSource::Url {
                    url: "multi.woff2".to_string(),
                    formats: vec![FontFormat::Woff2],
                    techs: vec![FontTechnology::Variations, FontTechnology::Palettes],
                },
                FontSource::Local {
                    family: "FooBar".to_string()
                }
            ]
        );
    }

    #[test]
    fn test_parse_font_face_with_named_weight_keyword() {
        let input = r#"
            @font-face {
                font-family: KeywordWeight;
                font-weight: bold;
                src: url("a.otf") format("opentype");
            }
        "#;
        let at_rule = parse_font_face_at_rule(input);
        let ff = FontFaceRule::from_rule(&at_rule);

        assert_eq!(ff.family, "KeywordWeight");
        assert_eq!(ff.weight, FontWeight::Single(700)); // "bold" stored as 700
        assert_eq!(
            ff.sources,
            vec![FontSource::Url {
                url: "a.otf".to_string(),
                formats: vec![FontFormat::OpenType],
                techs: vec![],
            }]
        );
    }

    #[test]
    fn test_parse_font_face_cyrillic_ext() {
        let input = r#"
            @font-face {
                font-family: 'Inter';
                font-style: italic;
                font-weight: 100 900;
                font-display: swap;
                src: url(https://fonts.gstatic.com/s/inter/v20/UcCm3FwrK3iLTcvnUwkT9nA2.woff2) format('woff2');
                unicode-range: U+0460-052F, U+1C80-1C8A, U+20B4, U+2DE0-2DFF, U+A640-A69F, U+FE2E-FE2F;
            }
        "#;
        let at_rule = parse_font_face_at_rule(input);
        let ff = FontFaceRule::from_rule(&at_rule);

        assert_eq!(ff.family, "Inter");
        assert_eq!(ff.style, FontStyle::Italic);
        assert_eq!(ff.weight, FontWeight::Range { min: 100, max: 900 });
        assert_eq!(ff.display, FontDisplay::Swap);
        assert_eq!(
            ff.sources,
            vec![FontSource::Url {
                url: "https://fonts.gstatic.com/s/inter/v20/UcCm3FwrK3iLTcvnUwkT9nA2.woff2"
                    .to_string(),
                formats: vec![FontFormat::Woff2],
                techs: vec![]
            }]
        );
        let ranges = &ff.unicode_range;
        assert_eq!(ranges.len(), 6);
        assert_eq!(ranges[0].start, 0x0460);
        assert_eq!(ranges[0].end, 0x052F);
        assert_eq!(ranges[2].start, 0x20B4);
        assert_eq!(ranges[2].end, 0x20B4);
    }

    #[test]
    fn test_parse_font_face_cyrillic() {
        let input = r#"
            @font-face {
                font-family: 'Inter';
                font-style: italic;
                font-weight: 100 900;
                font-display: swap;
                src: url(https://fonts.gstatic.com/s/inter/v20/UcCm3FwrK3iLTcvnUwAT9nA2.woff2) format('woff2');
                unicode-range: U+0301, U+0400-045F, U+0490-0491, U+04B0-04B1, U+2116;
            }
        "#;
        let at_rule = parse_font_face_at_rule(input);
        let ff = FontFaceRule::from_rule(&at_rule);

        assert_eq!(ff.family, "Inter");
        assert_eq!(ff.style, FontStyle::Italic);
        assert_eq!(ff.weight, FontWeight::Range { min: 100, max: 900 });
        assert_eq!(ff.display, FontDisplay::Swap);
        assert_eq!(
            ff.sources,
            vec![FontSource::Url {
                url: "https://fonts.gstatic.com/s/inter/v20/UcCm3FwrK3iLTcvnUwAT9nA2.woff2"
                    .to_string(),
                formats: vec![FontFormat::Woff2],
                techs: vec![]
            }]
        );
        let ranges = &ff.unicode_range;
        assert_eq!(ranges.len(), 5);
        assert_eq!(ranges[0].start, 0x0301);
        assert_eq!(ranges[0].end, 0x0301);
        assert_eq!(ranges[1].start, 0x0400);
        assert_eq!(ranges[1].end, 0x045F);
        assert_eq!(ranges[4].start, 0x2116);
        assert_eq!(ranges[4].end, 0x2116);
    }

    #[test]
    fn test_parse_font_face_greek_ext() {
        let input = r#"
            @font-face {
                font-family: 'Inter';
                font-style: italic;
                font-weight: 100 900;
                font-display: swap;
                src: url(https://fonts.gstatic.com/s/inter/v20/UcCm3FwrK3iLTcvnUwgT9nA2.woff2) format('woff2');
                unicode-range: U+1F00-1FFF;
            }
        "#;
        let at_rule = parse_font_face_at_rule(input);
        let ff = FontFaceRule::from_rule(&at_rule);

        assert_eq!(ff.family, "Inter");
        assert_eq!(ff.style, FontStyle::Italic);
        assert_eq!(ff.weight, FontWeight::Range { min: 100, max: 900 });
        assert_eq!(ff.display, FontDisplay::Swap);
        assert_eq!(
            ff.sources,
            vec![FontSource::Url {
                url: "https://fonts.gstatic.com/s/inter/v20/UcCm3FwrK3iLTcvnUwgT9nA2.woff2"
                    .to_string(),
                formats: vec![FontFormat::Woff2],
                techs: vec![]
            }]
        );
        let ranges = &ff.unicode_range;
        assert_eq!(ranges.len(), 1);
        assert_eq!(ranges[0].start, 0x1F00);
        assert_eq!(ranges[0].end, 0x1FFF);
    }

    #[test]
    fn test_parse_font_face_greek() {
        let input = r#"
            @font-face {
                font-family: 'Inter';
                font-style: italic;
                font-weight: 100 900;
                font-display: swap;
                src: url(https://fonts.gstatic.com/s/inter/v20/UcCm3FwrK3iLTcvnUwcT9nA2.woff2) format('woff2');
                unicode-range: U+0370-0377, U+037A-037F, U+0384-038A, U+038C, U+038E-03A1, U+03A3-03FF;
            }
        "#;
        let at_rule = parse_font_face_at_rule(input);
        let ff = FontFaceRule::from_rule(&at_rule);

        assert_eq!(ff.family, "Inter");
        assert_eq!(ff.style, FontStyle::Italic);
        assert_eq!(ff.weight, FontWeight::Range { min: 100, max: 900 });
        assert_eq!(ff.display, FontDisplay::Swap);
        assert_eq!(
            ff.sources,
            vec![FontSource::Url {
                url: "https://fonts.gstatic.com/s/inter/v20/UcCm3FwrK3iLTcvnUwcT9nA2.woff2"
                    .to_string(),
                formats: vec![FontFormat::Woff2],
                techs: vec![]
            }]
        );
        let ranges = &ff.unicode_range;
        assert_eq!(ranges.len(), 6);
        assert_eq!(ranges[0].start, 0x0370);
        assert_eq!(ranges[0].end, 0x0377);
        assert_eq!(ranges[3].start, 0x038C);
        assert_eq!(ranges[3].end, 0x038C);
        assert_eq!(ranges[5].start, 0x03A3);
        assert_eq!(ranges[5].end, 0x03FF);
    }

    #[test]
    fn test_parse_font_face_vietnamese() {
        let input = r#"
            @font-face {
                font-family: 'Inter';
                font-style: italic;
                font-weight: 100 900;
                font-display: swap;
                src: url(https://fonts.gstatic.com/s/inter/v20/UcCm3FwrK3iLTcvnUwsT9nA2.woff2) format('woff2');
                unicode-range: U+0102-0103, U+0110-0111, U+0128-0129, U+0168-0169, U+01A0-01A1, U+01AF-01B0, U+0300-0301, U+0303-0304, U+0308-0309, U+0323, U+0329, U+1EA0-1EF9, U+20AB;
            }
        "#;
        let at_rule = parse_font_face_at_rule(input);
        let ff = FontFaceRule::from_rule(&at_rule);

        assert_eq!(ff.family, "Inter");
        assert_eq!(ff.style, FontStyle::Italic);
        assert_eq!(ff.weight, FontWeight::Range { min: 100, max: 900 });
        assert_eq!(ff.display, FontDisplay::Swap);
        assert_eq!(
            ff.sources,
            vec![FontSource::Url {
                url: "https://fonts.gstatic.com/s/inter/v20/UcCm3FwrK3iLTcvnUwsT9nA2.woff2"
                    .to_string(),
                formats: vec![FontFormat::Woff2],
                techs: vec![]
            }]
        );
        let ranges = &ff.unicode_range;
        assert_eq!(ranges.len(), 13);
        assert_eq!(ranges[0].start, 0x0102);
        assert_eq!(ranges[0].end, 0x0103);
        assert_eq!(ranges[9].start, 0x0323);
        assert_eq!(ranges[9].end, 0x0323);
        assert_eq!(ranges[12].start, 0x20AB);
        assert_eq!(ranges[12].end, 0x20AB);
    }

    #[test]
    fn test_parse_font_face_with_ident_family_and_svg_format() {
        let input = r#"
            @font-face {
                font-family: IdentOnly Font;
                src: url("font.svg") format(svg);
            }
        "#;
        let at_rule = parse_font_face_at_rule(input);
        let ff = FontFaceRule::from_rule(&at_rule);

        assert_eq!(ff.family, "IdentOnly Font");
        assert_eq!(
            ff.sources,
            vec![FontSource::Url {
                url: "font.svg".to_string(),
                formats: vec![FontFormat::Svg],
                techs: vec![],
            }]
        );
    }

    #[test]
    fn test_parse_font_face_with_oblique_range_angle() {
        let input = r#"
            @font-face {
                font-family: ObliqueAngleRange;
                font-style: oblique 10deg 20deg;
                src: url("angle.otf");
            }
        "#;
        let at_rule = parse_font_face_at_rule(input);
        let ff = FontFaceRule::from_rule(&at_rule);

        assert_eq!(ff.family, "ObliqueAngleRange");
        assert_eq!(
            ff.style,
            FontStyle::Oblique(ObliqueStyle::Range(
                Angle {
                    value: 10.0,
                    unit: AngleUnit::Deg,
                },
                Angle {
                    value: 20.0,
                    unit: AngleUnit::Deg,
                }
            ))
        );
        assert_eq!(
            ff.sources,
            vec![FontSource::Url {
                url: "angle.otf".to_string(),
                formats: vec![],
                techs: vec![],
            }]
        );
    }

    #[test]
    fn test_parse_font_face_with_unicode_range_multiple_singles() {
        let input = r#"
            @font-face {
                font-family: UnicodeSingles;
                unicode-range: U+23A7,U+23AF,U+23B7;
                src: url("unic.ttf");
            }
        "#;
        let at_rule = parse_font_face_at_rule(input);
        let ff = FontFaceRule::from_rule(&at_rule);
        assert_eq!(ff.family, "UnicodeSingles");
        assert_eq!(
            ff.unicode_range,
            vec![
                UnicodeRange {
                    start: 0x23A7,
                    end: 0x23A7
                },
                UnicodeRange {
                    start: 0x23AF,
                    end: 0x23AF
                },
                UnicodeRange {
                    start: 0x23B7,
                    end: 0x23B7
                },
            ]
        );
    }

    #[test]
    fn test_parse_font_face_with_all_keywords_and_no_quotes() {
        let input = r#"
            @font-face {
                font-family: KWTest;
                font-style: italic;
                font-weight: normal;
                font-stretch: normal;
                font-display: auto;
                src: local(FooBar);
            }
        "#;
        let at_rule = parse_font_face_at_rule(input);
        let ff = FontFaceRule::from_rule(&at_rule);
        assert_eq!(ff.family, "KWTest");
        assert_eq!(ff.style, FontStyle::Italic);
        assert_eq!(ff.weight, FontWeight::Single(400)); // "normal" stored as 400
        assert_eq!(ff.stretch, FontStretch::Normal);
        assert_eq!(ff.display, FontDisplay::Auto);
        assert_eq!(
            ff.sources,
            vec![FontSource::Local {
                family: "FooBar".to_string()
            }]
        );
    }

    #[test]
    fn test_parse_font_face_empty_block() {
        let at_rule = AtRule {
            name: "font-face".into(),
            prelude: vec![],
            block: Some(AtRuleBlock::DeclarationList(vec![])),
        };
        let ff = FontFaceRule::from_rule(&at_rule);
        assert_eq!(ff.family, "");
        assert_eq!(ff.sources.len(), 0);
        assert_eq!(ff.style, FontStyle::default());
        assert_eq!(ff.weight, FontWeight::default());
        assert_eq!(ff.stretch, FontStretch::default());
        assert_eq!(ff.display, FontDisplay::default());
        assert_eq!(ff.unicode_range, vec![]);
    }
}
