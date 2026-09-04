use crate::LoadedFont;
use std::{collections::HashMap, sync::Arc};
use url::Url;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FontKey {
    Web { url: Url },
    Local { id: fontdb::ID },
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct FontCache {
    loaded: HashMap<FontKey, Arc<LoadedFont>>,
}

impl FontCache {
    pub fn new() -> Self {
        Self {
            loaded: HashMap::new(),
        }
    }
    pub fn get(&self, source: &FontKey) -> Option<Arc<LoadedFont>> {
        self.loaded.get(source).cloned()
    }

    pub fn insert(&mut self, source: FontKey, font: Arc<LoadedFont>) -> Arc<LoadedFont> {
        self.loaded.insert(source.clone(), font.clone());
        font
    }

    pub fn contains(&self, source: &FontKey) -> bool {
        self.loaded.contains_key(source)
    }

    pub fn clear(&mut self) {
        self.loaded.clear();
    }
}
