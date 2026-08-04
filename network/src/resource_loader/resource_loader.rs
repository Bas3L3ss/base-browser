use anyhow::{Result, anyhow};
use html::dom::{dom::Dom, node_type::NodeType, style_sheet_ref::StyleSheetRef};
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

    pub fn load_stylesheet(
        &self,
        dom: &Dom,
        stylesheet: &StyleSheetRef,
        base_url: &Url,
    ) -> anyhow::Result<String> {
        profile_scope!("Load StyleSheet");
        match stylesheet {
            StyleSheetRef::InlineStyle(node_id) => {
                let child = dom.nodes[*node_id].children[0];

                match &dom.nodes[child].ntype {
                    NodeType::Text(css) => Ok(css.clone()),
                    _ => Ok(String::new()),
                }
            }

            StyleSheetRef::StyleSheetLink(node_id) => {
                let href = match &dom.nodes[*node_id].ntype {
                    NodeType::Element(element) => element
                        .attributes
                        .href
                        .as_ref()
                        .ok_or_else(|| anyhow!("missing href"))?,
                    _ => return Err(anyhow!("expected element")),
                };

                let url = base_url.resolve(href)?;
                let response = self.http.get(&url)?;

                Ok(response.body_as_string())
            }
        }
    }
}
