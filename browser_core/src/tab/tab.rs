use anyhow::{ Ok, Result };
use url::Url;

use crate::{ browser::BrowserShared, document::document::Document };
use layout::{ Dimensions, LayoutEngine, Rect };

pub struct Tab<'a> {
    pub document: Option<Document<'a>>,
    // session_history: SessionHistory,

    // loading_state: LoadingState,

    // viewport: Viewport,
}

impl<'a> Tab<'a> {
    pub fn new() -> Self {
        Self { document: None }
    }

    pub fn navigate(&mut self, url: &'a Url, shared: &mut BrowserShared) -> Result<()> {
        let mut document = Document::new(&shared.ua_stylesheet, &url, &shared.resource_loader)?;

        LayoutEngine::layout(
            &mut document.layout_tree,
            Dimensions {
                content: Rect {
                    x: 0.0,
                    y: 0.0,
                    width: 800.0,
                    height: 600.0,
                },
                ..Default::default()
            },
            &document.computed_styles,
            &document.dom,
            &document.cssom,
            &mut shared.font_manager,
            &shared.resource_loader,
            &url
        );

        self.document = Some(document);

        Ok(())
    }

    pub fn render(&self) -> Result<()> {
        let _ = &self.document;
        Ok(())
    }
}
