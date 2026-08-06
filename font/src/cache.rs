use std::{collections::HashMap, sync::Arc};

use crate::{FontSourceRef, LoadedFont};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct FontCache {
    loaded: HashMap<FontSourceRef, Arc<LoadedFont>>,
}

impl FontCache {
    pub fn get(&self, source: &FontSourceRef) -> Option<Arc<LoadedFont>> {
        self.loaded.get(source).cloned()
    }

    pub fn insert(&mut self, source: FontSourceRef, font: Arc<LoadedFont>) -> Arc<LoadedFont> {
        self.loaded.insert(source.clone(), font.clone());
        font
    }

    pub fn contains(&self, source: &FontSourceRef) -> bool {
        self.loaded.contains_key(source)
    }

    pub fn clear(&mut self) {
        self.loaded.clear();
    }
}
