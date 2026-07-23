use std::{format, sync::Arc, unimplemented};

use crate::{
    dns::{resolver::DnsResolver, response::LookupResult},
    http::response::HttpResponse,
    network::{connection::Connection, connection_factory::ConnectionFactory},
};
use anyhow::Result;
use rustls::{ClientConfig, RootCertStore};
use url::url::Url;

pub struct HttpClient {
    dns_resolver: DnsResolver,
    connection_factory: ConnectionFactory,
}

impl HttpClient {
    pub fn new() -> Self {
        let dns_resolver = DnsResolver::new();
        let root_store = RootCertStore::from_iter(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
        let config = ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();

        let config = Arc::new(config);
        let connection_factory = ConnectionFactory::new(config);

        HttpClient {
            dns_resolver,
            connection_factory,
        }
    }

    pub fn get(&self, url: &Url) -> Result<HttpResponse> {
        let lookup_result: LookupResult = match self.dns_resolver.resolve(&url) {
            Ok(ip) => ip,
            Err(err) => {
                eprintln!("DNS lookup failed: {err}");
                return Err(err.into());
            }
        };

        let mut conn: Box<dyn Connection> = self
            .connection_factory
            .connect(url, lookup_result.addresses[0].ip)?;

        let host = url.host.to_string();
        let path = if url.path.is_empty() { "/" } else { &url.path };
        let request = format!(
            "GET {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
            path, host
        ); // TODO: HttpRequest::get(url)

        // Write request
        conn.write(request.as_bytes())?;

        // Read response
        let response_bytes = conn.read()?;
        let response = HttpResponse::from_bytes(&response_bytes)?;

        Ok(response)
    }

    pub fn post(
        &self,
        _url: &Url,
        _body: &[u8],
        _headers: Option<Vec<(String, String)>>,
    ) -> Result<HttpResponse> {
        unimplemented!("HTTP POST implementation");
    }

    pub fn put(
        &self,
        _url: &Url,
        _body: &[u8],
        _headers: Option<Vec<(String, String)>>,
    ) -> Result<HttpResponse> {
        unimplemented!("HTTP PUT implementation");
    }

    pub fn delete(
        &self,
        _url: &Url,
        _headers: Option<Vec<(String, String)>>,
    ) -> Result<HttpResponse> {
        unimplemented!("HTTP DELETE implementation");
    }

    pub fn head(
        &self,
        _url: &Url,
        _headers: Option<Vec<(String, String)>>,
    ) -> Result<HttpResponse> {
        unimplemented!("HTTP HEAD implementation");
    }
}
