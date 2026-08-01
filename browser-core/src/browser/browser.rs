use std::sync::Arc;

use anyhow::Result;
use css::cssom::cssom::CssOm;
use url::url::Url;

use crate::tab::tab::Tab;

pub struct Browser {
    pub shared: Arc<BrowserShared>,
    pub tabs: Vec<Tab>,
}

pub struct BrowserShared {
    // CSS
    pub ua_stylesheet: CssOm,
    // Networking
    // dns_cache: DnsCache,
    // http_cache: HttpCache,
    // cookie_store: CookieStore,

    // Resources
    // image_cache: ImageCache,
    // font_manager: FontManager,

    // History
    // history: HistoryDatabase,

    // Configuration
    // settings: BrowserSettings,
}

impl BrowserShared {
    pub fn new() -> Self {
        Self {
            ua_stylesheet: CssOm::default(),
        }
    }
}

impl Default for BrowserShared {
    fn default() -> Self {
        Self::new()
    }
}

impl Browser {
    pub fn new() -> Self {
        Self {
            shared: Arc::new(BrowserShared::new()),
            tabs: Vec::new(),
        }
    }

    pub fn open_tab(&mut self, url: Url) -> Result<()> {
        let tab = Tab::new(self.shared.clone(), url)?;
        self.tabs.push(tab);
        Ok(())
    }

    pub fn run(&self) -> Result<()> {
        for tab in &self.tabs {
            tab.render()?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::assert_eq;

    use input_parser::traits::Parse;

    use super::*;

    #[test]
    fn browser_can_create_and_open_tabs() {
        let mut browser = Browser::new();
        let url = Url::parse("https://example.com").unwrap().value;

        browser.open_tab(url.clone()).unwrap();

        assert_eq!(browser.tabs.len(), 1);
        assert_eq!(browser.tabs[0].document.url, url);
    }
}
