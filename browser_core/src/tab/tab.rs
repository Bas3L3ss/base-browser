use std::sync::Arc;

use anyhow::Result;
use url::Url;

use crate::{browser::BrowserShared, document::document::Document};

pub struct Tab {
    pub shared: Arc<BrowserShared>,

    pub document: Document,
    // session_history: SessionHistory,

    // loading_state: LoadingState,

    // viewport: Viewport,
}

impl Tab {
    pub fn new(shared: Arc<BrowserShared>, url: Url) -> Result<Self> {
        let document = Document::new(&shared.ua_stylesheet, url)?;
        Ok(Self { shared, document })
    }

    pub fn render(&self) -> Result<()> {
        let _ = &self.document;
        Ok(())
    }
}
