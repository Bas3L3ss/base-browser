use std::{eprintln, sync::Arc};

use anyhow::Result;
use url::url::Url;

use crate::{browser::BrowserShared, tab::tab::Tab};

pub struct Browser {
    pub shared: Arc<BrowserShared>,
    pub tabs: Vec<Tab>,
}

impl Browser {
    pub fn new() -> Self {
        Self {
            shared: Arc::new(BrowserShared::new().unwrap_or_else(|e| {
                eprintln!("Failed to initialize BrowserShared: {}", e);
                std::process::exit(1);
            })),

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
