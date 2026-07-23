use rustls::pki_types::ServerName;
use rustls::{ClientConfig, ClientConnection};

use std::io::Write;
use std::io::{ErrorKind, Read};
use std::net::TcpStream;

use crate::connection::connection::Connection;

pub struct TlsConnection {
    pub tcp: TcpStream,
    pub tls: ClientConnection,
}
impl TlsConnection {
    pub fn connect(
        ip: std::net::IpAddr,
        port: u16,
        host: &str,
        config: std::sync::Arc<ClientConfig>,
    ) -> anyhow::Result<Self> {
        let server_name = ServerName::try_from(host.to_string())?;
        let tls = ClientConnection::new(config, server_name)?;
        let tcp = TcpStream::connect((ip, port))?;

        let mut conn = Self { tcp, tls };

        conn.handshake()?;

        Ok(conn)
    }

    fn handshake(&mut self) -> anyhow::Result<()> {
        while self.tls.is_handshaking() {
            self.tls.complete_io(&mut self.tcp)?; // TODO: change this to even-loop
        }
        Ok(())
    }
}

impl Connection for TlsConnection {
    fn write(&mut self, bytes: &[u8]) -> anyhow::Result<()> {
        // Write data into TLS connection
        let written = self.tls.writer().write(bytes)?;
        // Complete IO to flush the data to the underlying TCP stream
        let _ = self.tls.complete_io(&mut self.tcp)?;
        // Optionally check that all bytes were written
        if written < bytes.len() {
            anyhow::bail!("Not all bytes were written to TLS stream");
        }
        Ok(())
    }

    fn read(&mut self) -> anyhow::Result<Vec<u8>> {
        // Pre-allocate buffer with 4096 bytes, which is a common TLS record size,
        // to efficiently handle typical TLS record reads and reduce early reallocations.
        let mut buf = Vec::with_capacity(4096);
        loop {
            // complete_io will handle the actual read from the TCP stream and decrypt
            let io_result = self.tls.complete_io(&mut self.tcp);

            match io_result {
                Ok((_rd, _wr)) => {
                    let mut tmp = [0u8; 4096];
                    match self.tls.reader().read(&mut tmp) {
                        Ok(0) => break, // No more data
                        Ok(n) => buf.extend_from_slice(&tmp[..n]),
                        Err(ref e) if e.kind() == ErrorKind::WouldBlock => continue,
                        Err(e) => return Err(e.into()),
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => continue,
                Err(e) => return Err(e.into()),
            }
        }
        Ok(buf)
    }
}
