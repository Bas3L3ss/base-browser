use std::io::{Read, Write};
use std::net::TcpStream;

use crate::network::connection::Connection;

pub struct TcpConnection {
    pub stream: TcpStream,
}

impl TcpConnection {
    pub fn connect(ip: std::net::IpAddr, port: u16) -> anyhow::Result<Self> {
        let stream = TcpStream::connect((ip, port))?;
        Ok(Self { stream })
    }
}

impl Connection for TcpConnection {
    fn write(&mut self, bytes: &[u8]) -> anyhow::Result<()> {
        self.stream.write_all(bytes)?;
        Ok(())
    }

    fn read(&mut self) -> anyhow::Result<Vec<u8>> {
        let mut buf = Vec::new();
        self.stream.read_to_end(&mut buf)?;
        Ok(buf)
    }
}
