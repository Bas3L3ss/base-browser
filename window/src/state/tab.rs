use std::sync::Arc;

use anyhow::{ Ok, Result };
use css::CssOm;
use font::FontManager;
use network::ResourceLoader;
use url::Url;
use layout::{ LayoutEngine };
use winit::{ dpi::PhysicalSize, window::Window };

use crate::Document;

pub type TabId = usize;
#[derive(Debug, Clone)]
pub struct Tab {
    pub tab_id: TabId,
    pub document: Option<Document>,
    // session_history: SessionHistory,

    // loading_state: LoadingState,

    // viewport: Viewport,
}

impl Tab {
    pub fn new(tab_id: usize) -> Self {
        Self { document: None, tab_id }
    }

    pub fn navigate(
        &mut self,
        url: &Url,
        ua_stylesheet: &CssOm,
        resource_loader: &ResourceLoader,
        font_manager: &mut FontManager,
        window: &Window
    ) -> Result<()> {
        let mut document = Document::new(&ua_stylesheet, &url, &resource_loader)?;
        dbg!(&document.cssom.rules[4]);
        LayoutEngine::layout(
            &mut document.layout_tree,
            &document.computed_styles,
            &document.dom,
            &document.cssom,
            font_manager,
            &resource_loader,
            &url,
            window.inner_size()
        );

        self.document = Some(document);

        Ok(())
    }
    pub fn rerender(
        &mut self,
        resource_loader: &ResourceLoader,
        font_manager: &mut FontManager,
        size: PhysicalSize<u32>
    ) -> Result<()> {
        if let Some(document) = self.document.as_mut() {
            LayoutEngine::layout(
                &mut document.layout_tree,
                &document.computed_styles,
                &document.dom,
                &document.cssom,
                font_manager,
                &resource_loader,
                &document.url,
                size
            );
        }

        Ok(())
    }
}
