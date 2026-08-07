pub mod decoder;
pub mod matcher;
pub mod support;

use std::{collections::HashMap, sync::Arc, todo};

use css::FontFaceRule;
use network::ResourceLoader;
use url::Url;
// TODO: add matching and optimization for unicode range
use crate::{
    FontCache, FontRequest, LoadedFont,
    decoder::FontDecoder,
    matcher::FontMatcher,
    support::{FontFileFormat, FontSupport},
};

#[derive(Debug, Clone, PartialEq, Default)]

pub struct FontManager {
    cache: FontCache,
}

impl FontManager {
    pub fn resolve_font(
        &mut self,
        font_faces: &HashMap<String, Vec<FontFaceRule>>,
        request: &FontRequest,
        base_url: &Url,
        loader: &ResourceLoader,
    ) -> Arc<LoadedFont> {
        for family in request.families {
            if let Some(font_faces) = font_faces.get(family) {
                if let Some(font) = self.resolve_web_font(font_faces, request, base_url, loader) {
                    return font;
                }
            }

            if let Some(font) = self.resolve_system_font(family, request) {
                return font;
            }

            if let Some(font) = self.resolve_generic_font(family, request) {
                return font;
            }
        }

        self.default_font()
    }

    fn load_local(&mut self, local_name: &str) -> Option<Arc<LoadedFont>> {
        todo!()
    }

    fn load_url(
        &mut self,
        href: &str,
        base_url: &Url,
        formats: &[css::FontFormat],
        techs: &[css::FontTechnology],
        loader: &ResourceLoader,
    ) -> Option<Arc<LoadedFont>> {
        if !FontSupport::supports_tech(techs) {
            return None;
        }
        // let url = base_url.resolve(href).ok()?;
        let format = FontSupport::detect_format_from_css(formats, href);

        if !FontSupport::supports_format(format) {
            return None;
        }

        let url = base_url.resolve(href).ok()?;
        let mut bytes = loader.load_bytes(&url).ok()?;

        // Only if detect_format couldn't determine it.
        let format = if format == FontFileFormat::Unknown {
            FontSupport::detect_format_from_bytes(&bytes)
        } else {
            format
        };

        let bytes = match format {
            FontFileFormat::TrueType | FontFileFormat::OpenType => bytes,

            FontFileFormat::Woff => FontDecoder::decode_woff(&bytes)?,

            FontFileFormat::Woff2 => FontDecoder::decode_woff2(&mut bytes)?,

            _ => return None,
        };

        LoadedFont::from_bytes(bytes)
    }

    fn load_bytes(&self, _url: &Url) -> anyhow::Result<Vec<u8>> {
        todo!()
    }

    fn decode_font(
        &self,
        _bytes: Vec<u8>,
        _formats: &[css::FontFormat],
    ) -> anyhow::Result<Vec<u8>> {
        todo!()
    }
    fn load_font_face(
        &mut self,
        source: &css::rule::FontSource,
        base_url: &Url,
        loader: &ResourceLoader,
    ) -> Option<Arc<LoadedFont>> {
        match source {
            css::FontSource::Local { family } => self.load_local(family),
            css::FontSource::Url {
                url,
                formats,
                techs,
            } => self.load_url(url, base_url, formats, techs, loader),
        }
    }

    fn resolve_web_font(
        &mut self,
        font_faces: &[FontFaceRule],
        request: &FontRequest,
        base_url: &Url,
        loader: &ResourceLoader,
    ) -> Option<Arc<LoadedFont>> {
        if let Some(face) = FontMatcher::match_font_face(font_faces, request) {
            for source in &face.sources {
                if let Some(loaded) = self.load_font_face(source, base_url, loader) {
                    return Some(loaded);
                }
            }
        }
        None
    }

    fn resolve_system_font(
        &mut self,
        family: &str,
        request: &FontRequest,
    ) -> Option<Arc<LoadedFont>> {
        todo!()
    }

    fn resolve_generic_font(
        &mut self,
        family: &str,
        request: &FontRequest,
    ) -> Option<Arc<LoadedFont>> {
        todo!()
    }

    fn default_font(&mut self) -> Arc<LoadedFont> {
        todo!()
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
