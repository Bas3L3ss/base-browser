use std::{net::IpAddr, sync::Arc};

use rustls::ClientConfig;

use crate::{
    network::{
        connection::Connection, tcp_connection::TcpConnection, tls_connection::TlsConnection,
    },
    url::url::{Scheme, Url},
};

pub struct ConnectionFactory {
    config: Arc<ClientConfig>,
}

impl ConnectionFactory {
    pub fn new(config: Arc<ClientConfig>) -> Self {
        Self { config }
    }
    pub fn connect(&self, url: &Url, ip: IpAddr) -> anyhow::Result<Box<dyn Connection>> {
        match url.scheme {
            Scheme::Http => Ok(Box::new(TcpConnection::connect(ip, url.port)?)),

            Scheme::Https => Ok(Box::new(TlsConnection::connect(
                ip,
                url.port,
                &url.host.to_string(),
                self.config.clone(),
            )?)),
        }
    }
}
