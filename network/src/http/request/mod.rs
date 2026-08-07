pub(crate) mod header;
pub(crate) mod method;
pub(crate) mod version;

use std::{format, todo};

use url::Url;

use crate::{http::header::HttpHeaders, method::HttpMethod, version::HttpVersion};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpRequest {
    pub method: HttpMethod,
    pub target: String,
    pub version: HttpVersion,
    pub headers: HttpHeaders,
    pub body: Vec<u8>,
}

impl HttpRequest {
    pub fn get(url: &Url) -> Self {
        let mut target = if url.path.is_empty() {
            "/".to_string()
        } else {
            url.path.clone()
        };

        if let Some(q) = &url.query {
            target.push('?');
            target.push_str(q);
        }

        let mut headers = HttpHeaders::new();

        // Host header: use domain[:port] (add :port only if not default)
        let mut host_value = format!("{}", url.host);

        host_value = format!("{}:{}", host_value, url.port);
        headers.set("Host", &host_value);
        headers.set("User-Agent", "BaseBrowser/0.1");
        headers.set("Accept", "*/*");
        headers.set("Connection", "close");

        HttpRequest {
            method: HttpMethod::GET,
            target,
            version: HttpVersion::Http11,
            headers,
            body: Vec::new(),
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut request = String::new();

        // Request line using Display for method and version
        request.push_str(&format!(
            "{} {} {}\r\n",
            self.method, self.target, self.version
        ));

        // Headers
        for header in self.headers.iter() {
            request.push_str(&format!("{}: {}\r\n", header.name, header.value));
        }

        if !self.body.is_empty() && self.headers.get("Content-Length").is_none() {
            request.push_str(&format!("Content-Length: {}\r\n", self.body.len()));
        }

        // End headers
        request.push_str("\r\n");

        let mut bytes = request.into_bytes();

        // Body (if present)
        bytes.extend_from_slice(&self.body);

        bytes
    }

    pub fn post(_url: &Url) -> Self {
        todo!()
    }
    pub fn put(_url: &Url) -> Self {
        todo!()
    }
    pub fn patch(_url: &Url) -> Self {
        todo!()
    }
    pub fn delete(_url: &Url) -> Self {
        todo!()
    }
    pub fn head(_url: &Url) -> Self {
        todo!()
    }
    pub fn options(_url: &Url) -> Self {
        todo!()
    }
    pub fn connect(_url: &Url) -> Self {
        todo!()
    }
    pub fn trace(_url: &Url) -> Self {
        todo!()
    }
}
