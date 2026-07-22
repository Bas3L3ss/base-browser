use std::net::{SocketAddr, UdpSocket};
use std::{net::IpAddr, vec};

use anyhow::{Result, anyhow, bail};

use crate::dns::enums::{QueryClass, QueryType};
use crate::dns::header::Header;
use crate::dns::packet::DnsPacket;
use crate::dns::question::DnsQuestion;
use crate::url::url::Url;

const GOOGLE_PUBLIC_DNS: ([u8; 4], u16) = ([8, 8, 8, 8], 53);
const GOOGLE_ALTERNATE_DNS: ([u8; 4], u16) = ([8, 8, 4, 4], 53);
pub struct DnsClient {}

impl DnsClient {
    pub fn lookup(url: &Url) -> Result<IpAddr> {
        let header = Header::new(0, 0, 1, 0, 0, 0);

        let host_str = match &url.host {
            crate::url::url::Host::Domain(d) => d.clone(),
            crate::url::url::Host::Ip(ip) => ip.to_string(),
        };
        let question = DnsQuestion::new(host_str, QueryType::default(), QueryClass::default());

        let packet: DnsPacket = DnsPacket::new(header, vec![question]);

        let socket = UdpSocket::bind("0.0.0.0:0")
            .map_err(|e| anyhow::anyhow!("Failed to bind UDP socket: {e}"))?;

        let primary_addr = SocketAddr::from(GOOGLE_PUBLIC_DNS);
        let alt_addr = SocketAddr::from(GOOGLE_ALTERNATE_DNS);
        let msg = packet.to_bytes();

        socket
            .send_to(&msg, primary_addr)
            .map_err(|e| anyhow::anyhow!("OS failed to send data: {e}"))?;

        // Use a 512-byte buffer, which is the maximum size of a DNS message over UDP per RFC 1035 (standard, non-EDNS).
        // This ensures we can receive any valid DNS response without truncation for classic DNS packets.
        let mut buf = [0u8; 512];
        let (len, from) = socket.recv_from(&mut buf)?;

        Ok(IpAddr::V4(std::net::Ipv4Addr::new(1, 2, 3, 4)))
    }
}
