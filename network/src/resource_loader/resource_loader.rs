use anyhow::{Ok, Result};
use url::url::Url;
use utils::profile_scope;

use crate::http::client::HttpClient;

pub struct ResourceLoader {
    http: HttpClient,
}

impl ResourceLoader {
    pub fn new(http: HttpClient) -> Self {
        ResourceLoader { http }
    }

    pub fn load_html(&self, url: &Url) -> Result<String> {
        profile_scope!("Load HTML");
        let response = self.http.get(url)?;
        Ok(response.body_as_string())
    }

    pub fn load_css(&self, base_url: &Url, href: &String) -> Result<String> {
        profile_scope!("Load CSS");
        let url = base_url.resolve(href)?;
        let response = self.http.get(&url)?;
        Ok(response.body_as_string())
    }

    pub fn load_bytes(&self, url: &Url) -> Result<Vec<u8>> {
        profile_scope!("Load resource");
        let response = self.http.get(url)?;
        Ok(response.body)
    }
}
