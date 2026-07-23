use std::net::{SocketAddr, UdpSocket};
use std::time::Duration;

use anyhow::{Result, anyhow};
use rand::RngExt;
use url::url::{Host, Url};

use crate::dns::enums::Rcode;
use crate::dns::enums::{QueryClass, QueryType};
use crate::dns::flag::Flag;
use crate::dns::header::Header;
use crate::dns::packet::DnsPacket;
use crate::dns::question::DnsQuestion;
use crate::dns::response::LookupResult;

const GOOGLE_PUBLIC_DNS: ([u8; 4], u16) = ([8, 8, 8, 8], 53);
const GOOGLE_ALTERNATE_DNS: ([u8; 4], u16) = ([8, 8, 4, 4], 53);

pub struct DnsResolver {}

impl DnsResolver {
    pub fn new() -> Self {
        DnsResolver {}
    }
    pub fn resolve(&self, url: &Url) -> Result<LookupResult> {
        let mut rng = rand::rng();
        let header_id: u16 = rng.random();

        let header = Header::new(header_id, Flag::new(), 1, 0, 0, 0);

        let host_str = match &url.host {
            Host::Domain(d) => d.clone(),
            Host::Ip(ip) => ip.to_string(),
        };
        let question = DnsQuestion::new(host_str, QueryType::default(), QueryClass::default());

        let query_packet = DnsPacket::new(header, vec![question]);
        let msg = query_packet.to_bytes();

        let socket =
            UdpSocket::bind("0.0.0.0:0").map_err(|e| anyhow!("Failed to bind UDP socket: {e}"))?;
        socket
            .set_read_timeout(Some(Duration::from_secs(10)))
            .map_err(|e| anyhow!("Failed to set socket timeout: {e}"))?;

        let servers = [
            SocketAddr::from(GOOGLE_PUBLIC_DNS),
            SocketAddr::from(GOOGLE_ALTERNATE_DNS),
        ];

        let mut last_error = None;
        for (i, server) in servers.iter().enumerate() {
            socket
                .send_to(&msg, *server)
                .map_err(|e| anyhow!("OS failed to send data to DNS server {}: {e}", i))?;

            let mut buf = [0u8; 512];
            let recv_result = socket.recv_from(&mut buf);

            let (len, _from) = match recv_result {
                Ok(res) => res,
                Err(e) => {
                    eprintln!(
                        "Timeout or error receiving DNS response from server {}: {:?}",
                        i, e
                    );
                    last_error = Some(anyhow!("Failed to receive DNS response: {:?}", e));
                    continue;
                }
            };

            let response_packet = match DnsPacket::from_bytes(&buf[..len]) {
                Ok(packet) => packet,
                Err(e) => {
                    eprintln!("Failed to parse DNS response from server {}: {:?}", i, e);
                    last_error = Some(anyhow!("Failed to parse DNS response: {:?}", e));
                    continue;
                }
            };

            // Validation: Only trust responses with matching ID
            if response_packet.header.id != header_id {
                eprintln!(
                    "DNS response from server {} had mismatched ID: expected {}, got {}. Ignoring.",
                    i, header_id, response_packet.header.id
                );
                last_error = Some(anyhow!("Mismatched DNS response ID"));
                continue;
            }

            let rcode = response_packet.header.flag.response_code;
            if rcode == Rcode::NoError {
                if let Some(addresses) = response_packet.extract_addresses() {
                    return Ok(LookupResult::new(addresses));
                } else {
                    last_error = Some(anyhow!("No valid A/AAAA record in response"));
                    continue;
                }
            } else if rcode == Rcode::NXDomain && i == 0 {
                eprintln!(
                    "Primary DNS returned NXDomain (domain does not exist). Not retrying with alternate."
                );
                return Err(anyhow!(
                    "NXDomain: domain does not exist (got from primary DNS)"
                ));
            } else {
                eprintln!(
                    "DNS server {} returned error: {:?}{}",
                    i,
                    rcode,
                    if i == 0 {
                        " (trying alternate DNS...)"
                    } else {
                        " (giving up)"
                    }
                );
                if i == servers.len() - 1 {
                    return Err(anyhow!("DNS error from alternate: {:?}", rcode));
                }
                // fallback to next iteration for alternate
            }
        }

        Err(last_error.unwrap_or_else(|| {
            anyhow!(
                "No A or AAAA record with valid IP address found in DNS response from all servers"
            )
        }))
    }
}
