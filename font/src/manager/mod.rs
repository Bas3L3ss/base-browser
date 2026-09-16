pub mod decoder;
pub mod default;
pub mod matcher;
pub mod support;

use std::{ collections::HashMap, sync::Arc };

use crate::{
    FontCache,
    FontKey,
    FontRequest,
    LoadedFont,
    decoder::FontDecoder,
    default::{ DEFAULT_MONO, DEFAULT_SANS, DEFAULT_SERIF },
    matcher::FontMatcher,
    support::{ FontFileFormat, FontSupport },
};
use css::FontFaceRule;
use network::ResourceLoader;
use url::Url;

#[derive(Clone)]
pub struct FontManager {
    pub cache: FontCache,
    pub font_db: fontdb::Database,
    pub cur_id: u16,
}

impl FontManager {
    pub fn new() -> Self {
        let mut font_db = fontdb::Database::new();
        font_db.load_system_fonts();

        Self {
            font_db,
            cache: FontCache::new(),
            cur_id: 0,
        }
    }

    pub fn resolve_font(
        &mut self,
        font_faces: &HashMap<String, Vec<FontFaceRule>>,
        request: &FontRequest,
        base_url: &Url,
        loader: &ResourceLoader
    ) -> (Arc<LoadedFont>, FontKey) {
        for family in request.families {
            if let Some(font_faces) = font_faces.get(family) {
                if
                    let Some((font, key)) = self.resolve_web_font(
                        font_faces,
                        request,
                        base_url,
                        loader
                    )
                {
                    return (font, key);
                }
            }

            if let Some((font, key)) = self.resolve_system_font(family, request) {
                return (font, key);
            }

            if let Some((font, key)) = self.resolve_generic_font(family, request) {
                return (font, key);
            }
        }

        self.default_font()
    }

    fn load_cached(
        &mut self,
        key: FontKey,
        mut load: impl FnMut(u16) -> Option<LoadedFont>
    ) -> Option<(Arc<LoadedFont>, FontKey)> {
        if let Some(font) = self.cache.get(&key) {
            return Some((font, key.clone()));
        }

        let id = self.cur_id;
        let font = Arc::new(load(id)?);
        self.cur_id = self.cur_id.wrapping_add(1);

        self.cache.insert(key.clone(), font.clone());

        Some((font, key))
    }

    fn load_system_face(&mut self, id: fontdb::ID) -> Option<(Arc<LoadedFont>, FontKey)> {
        let key = FontKey::Local { id };

        let face_info = {
            let face = self.font_db.face(id);

            // Now find the source path if present
            face.and_then(|face| {
                match &face.source {
                    fontdb::Source::File(path) => Some(path.clone()),
                    _ => None,
                }
            })
        };

        self.load_cached(key, |font_id| {
            let source = face_info.as_ref()?;
            let bytes = std::fs::read(source).ok()?;
            Some(LoadedFont::from_bytes(bytes, font_id).ok()?)
        })
    }

    fn load_url(
        &mut self,
        href: &str,
        base_url: &Url,
        formats: &[css::FontFormat],
        techs: &[css::FontTechnology],
        loader: &ResourceLoader
    ) -> Option<(Arc<LoadedFont>, FontKey)> {
        let url = base_url.resolve(href).ok()?;
        let key = FontKey::Web { url: url.clone() };

        self.load_cached(key, |font_id| {
            if !FontSupport::supports_tech(techs) {
                return None;
            }
            let format = FontSupport::detect_format_from_css(formats, href);

            if !FontSupport::supports_format(format) {
                return None;
            }

            let bytes = loader.load_bytes(&url).ok()?;

            // Only if detect_format couldn't determine it.
            let format = if format == FontFileFormat::Unknown {
                FontSupport::detect_format_from_bytes(&bytes)
            } else {
                format
            };

            let bytes = match format {
                FontFileFormat::TrueType | FontFileFormat::OpenType => bytes,
                FontFileFormat::Woff => FontDecoder::decode_woff(&bytes)?,
                FontFileFormat::Woff2 => FontDecoder::decode_woff2(&bytes)?,
                _ => {
                    return None;
                }
            };

            Some(LoadedFont::from_bytes(bytes, font_id).ok()?)
        })
    }

    fn load_local(
        &mut self,
        family: &str,
        request: &FontRequest
    ) -> Option<(Arc<LoadedFont>, FontKey)> {
        let id = FontMatcher::match_system_face(self.font_db.faces(), family, request)?;
        self.load_system_face(id)
    }

    fn load_font_face(
        &mut self,
        source: &css::rule::FontSource,
        base_url: &Url,
        loader: &ResourceLoader,
        request: &FontRequest
    ) -> Option<(Arc<LoadedFont>, FontKey)> {
        match source {
            css::FontSource::Local { family } => self.load_local(family, request),
            css::FontSource::Url { url, formats, techs } =>
                self.load_url(url, base_url, formats, techs, loader),
        }
    }

    fn resolve_web_font(
        &mut self,
        font_faces: &[FontFaceRule],
        request: &FontRequest,
        base_url: &Url,
        loader: &ResourceLoader
    ) -> Option<(Arc<LoadedFont>, FontKey)> {
        if let Some(face) = FontMatcher::match_font_face_rule(font_faces, request) {
            for source in &face.sources {
                if let Some((loaded, key)) = self.load_font_face(source, base_url, loader, request) {
                    return Some((loaded, key));
                }
            }
        }
        None
    }

    fn resolve_system_font(
        &mut self,
        family: &str,
        request: &FontRequest
    ) -> Option<(Arc<LoadedFont>, FontKey)> {
        let id = FontMatcher::match_system_face(self.font_db.faces(), family, request)?;
        self.load_system_face(id)
    }

    fn resolve_generic_font(
        &mut self,
        family: &str,
        request: &FontRequest
    ) -> Option<(Arc<LoadedFont>, FontKey)> {
        let fallbacks = match family {
            "serif" => DEFAULT_SERIF,
            "sans-serif" => DEFAULT_SANS,
            "monospace" => DEFAULT_MONO,

            // CSS generic families
            "system-ui" => DEFAULT_SANS,
            "ui-serif" => DEFAULT_SERIF,
            "ui-sans-serif" => DEFAULT_SANS,
            "ui-monospace" => DEFAULT_MONO,

            // Temporary fallbacks
            "cursive" => DEFAULT_SANS,
            "fantasy" => DEFAULT_SANS,
            "math" => DEFAULT_SERIF,

            _ => {
                return None;
            }
        };

        for family in fallbacks {
            if let Some(id) = FontMatcher::match_system_face(self.font_db.faces(), family, request) {
                if let Some((font, key)) = self.load_system_face(id) {
                    return Some((font, key));
                }
            }
        }
        None
    }

    fn default_font(&mut self) -> (Arc<LoadedFont>, FontKey) {
        // First search fonts that match one of the default sans families.
        let mut found: Option<(Arc<LoadedFont>, FontKey)> = None;
        let face_opt = self.font_db
            .faces()
            .find(|face| {
                face.families
                    .iter()
                    .any(|fam| {
                        DEFAULT_SANS.iter().any(|default| fam.0.eq_ignore_ascii_case(default))
                    })
            });
        if let Some(face) = face_opt {
            if let fontdb::Source::File(path) = &face.source {
                if let Ok(bytes) = std::fs::read(path) {
                    let font_id = self.cur_id;
                    if let Ok(loaded_font) = LoadedFont::from_bytes(bytes, font_id) {
                        let font = Arc::new(loaded_font);
                        let key = FontKey::Local { id: face.id };
                        found = Some((font, key));
                        self.cur_id = self.cur_id.wrapping_add(1);
                    }
                }
            }
        }

        // If found, return it.
        if let Some(result) = found {
            return result;
        }

        // Otherwise, fallback to any available font file.
        let any_font = self.font_db
            .faces()
            .find_map(|face| {
                if let fontdb::Source::File(path) = &face.source {
                    if let Ok(bytes) = std::fs::read(path) {
                        let font_id = self.cur_id;
                        if let Ok(loaded_font) = LoadedFont::from_bytes(bytes, font_id) {
                            let font = Arc::new(loaded_font);
                            let key = FontKey::Local { id: face.id };
                            self.cur_id = self.cur_id.wrapping_add(1);
                            return Some((font, key));
                        }
                    }
                }
                None
            })
            .expect("No usable system fonts found");

        any_font
    }
}

// resolve_web_font()
// │
// ├── match_font_face()
// │
// └── load_font_face()
//       │
//       ├── cache lookup
//       │
//       ├── load_local()
//       │
//       ├── load_url()
//       │
//       └── cache insert
