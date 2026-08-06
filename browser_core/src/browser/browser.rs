use std::eprintln;

use anyhow::Result;
use url::url::Url;

use crate::{browser::BrowserShared, tab::tab::Tab};

pub struct Browser {
    pub shared: BrowserShared,
    pub tabs: Vec<Tab>,
}

impl Browser {
    pub fn new() -> Self {
        Self {
            shared: BrowserShared::new().unwrap_or_else(|e| {
                eprintln!("Failed to initialize BrowserShared: {}", e);
                std::process::exit(1);
            }),

            tabs: Vec::new(),
        }
    }

    pub fn open_tab(&mut self, url: Url) -> Result<()> {
        let mut tab = Tab::new();
        tab.navigate(url, &mut self.shared)?;
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

    use super::*;
    use input_parser::Parse;
    use url::Url;

    #[test]
    fn test_open_tab_example_url() {
        let mut browser = Browser::new();
        let test_url = Url::parse("https://www.example.com").unwrap().value;
        let result = browser.open_tab(test_url);
        assert!(result.is_ok(), "Failed to open tab with example.com");
        assert_eq!(browser.tabs.len(), 1);
    }

    #[test]
    fn test_open_tab_local_apple_store() {
        let mut browser = Browser::new();
        let apple_url = Url::parse("http://127.0.0.1:5500/Apple-Store.html")
            .unwrap()
            .value;
        let result = browser.open_tab(apple_url);
        assert!(result.is_ok(), "Failed to open tab with Apple-Store.html");
        assert_eq!(browser.tabs.len(), 1);
    }
}
