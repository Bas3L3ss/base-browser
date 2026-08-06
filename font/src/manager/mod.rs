pub mod matcher;

use std::{collections::HashMap, sync::Arc, todo};

use css::FontFaceRule;
use url::Url;
// TODO: add matching and optimization for unicode range
use crate::{FontCache, FontRequest, LoadedFont, matcher::FontMatcher};

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
    ) -> Arc<LoadedFont> {
        for family in request.families {
            if let Some(font_faces) = font_faces.get(family) {
                if let Some(font) = self.resolve_web_font(font_faces, request, base_url) {
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

    fn load_local(local_name: &str) -> Option<Arc<LoadedFont>> {
        todo!()
    }

    fn load_url(url: &String, base_url: &Url) -> Option<Arc<LoadedFont>> {
        todo!()
    }

    fn load_font_face(
        &mut self,
        source: &css::rule::FontSource,
        base_url: &Url,
    ) -> Option<Arc<LoadedFont>> {
        match source {
            css::FontSource::Local { family } => Self::load_local(family),
            css::FontSource::Url {
                url,
                formats,
                techs,
            } => Self::load_url(url, base_url),
        }
    }

    fn resolve_web_font(
        &mut self,
        font_faces: &[FontFaceRule],
        request: &FontRequest,
        base_url: &Url,
    ) -> Option<Arc<LoadedFont>> {
        if let Some(face) = FontMatcher::match_font_face(font_faces, request) {
            for source in &face.sources {
                if let Some(loaded) = self.load_font_face(source, base_url) {
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
