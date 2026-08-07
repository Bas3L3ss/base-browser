use std::{eprintln, println};

use anyhow::Result;
use url::url::Url;

use crate::{browser::BrowserShared, tab::tab::Tab};

pub struct Browser<'a> {
    pub shared: BrowserShared,
    pub tabs: Vec<Tab<'a>>,
}

impl<'a> Browser<'a> {
    pub fn new() -> Self {
        Self {
            shared: BrowserShared::new().unwrap_or_else(|e| {
                eprintln!("Failed to initialize BrowserShared: {}", e);
                std::process::exit(1);
            }),

            tabs: Vec::new(),
        }
    }

    pub fn open_tab(&mut self, url: &'a Url) -> Result<()> {
        let mut tab = Tab::new();
        tab.navigate(&url, &mut self.shared)?;
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
    use core::panic;
    use std::assert_eq;

    use super::*;
    use input_parser::Parse;
    use std::sync::mpsc::channel;
    use std::thread;
    use url::Url;

    // Utility function to enforce a 3 second timeout on each test
    fn run_with_timeout<F>(f: F)
    where
        F: FnOnce() + std::panic::UnwindSafe + Send + 'static,
    {
        let (tx, rx) = channel();
        thread::spawn(move || {
            let _ = tx.send(std::panic::catch_unwind(f));
        });

        match rx.recv_timeout(std::time::Duration::from_secs(3)) {
            Ok(Ok(_)) => (),                            // Test finished successfully
            Ok(Err(e)) => std::panic::resume_unwind(e), // Test panicked, propagate
            Err(_) => panic!("Test timed out (took longer than 3 seconds)"),
        }
    }

    #[test]
    fn test_open_tab_example_url() {
        run_with_timeout(|| {
            let mut browser = Browser::new();
            let test_url = Url::parse("https://www.example.com").unwrap().value;
            let result = browser.open_tab(&test_url);
            assert!(result.is_ok(), "Failed to open tab with example.com");
            assert_eq!(browser.tabs.len(), 1);
        });
    }

    #[test]
    fn test_local_page() {
        run_with_timeout(|| {
            let mut browser = Browser::new();
            let apple_url = Url::parse("http://127.0.0.1:5500/index.html")
                .unwrap()
                .value;
            let result = browser.open_tab(&apple_url);
            assert!(result.is_ok(), "Failed to open tab with index.html");
            assert_eq!(browser.tabs.len(), 1);
        });
    }
}
