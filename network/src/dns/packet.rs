use std::{net::IpAddr, vec};

use anyhow::Result;

use crate::dns::{
    header::Header, question::DnsQuestion, record::DnsRecord, response::ResolvedAddress,
};

#[derive(Debug)]
pub struct DnsPacket {
    pub header: Header,
    pub questions: Vec<DnsQuestion>,
    pub answers: Vec<DnsRecord>,
    pub authorities: Vec<DnsRecord>,
    pub additional: Vec<DnsRecord>,
}

impl DnsPacket {
    pub fn new(header: Header, questions: Vec<DnsQuestion>) -> Self {
        DnsPacket {
            header,
            questions,
            answers: vec![],
            additional: vec![],
            authorities: vec![],
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let mut offset = 0;

        let header = Header::from_bytes(&bytes[offset..])?;
        offset += 12;

        let qdcount = header.num_questions as usize;
        let ancount = header.num_answers as usize;
        let nscount = header.num_authorities as usize;
        let arcount = header.num_additionals as usize;

        // 2. Parse the question section
        let mut questions = Vec::with_capacity(qdcount);
        for _ in 0..qdcount {
            let (question, consumed) = DnsQuestion::from_bytes(bytes, offset)?;
            questions.push(question);
            offset = consumed;
        }

        // 3. Parse the answer, authority, and additional sections
        let mut answers = Vec::with_capacity(ancount);
        for _ in 0..ancount {
            let (record, consumed) = DnsRecord::from_bytes(bytes, offset)?;
            answers.push(record);
            offset = consumed;
        }

        let mut authorities = Vec::with_capacity(nscount);
        for _ in 0..nscount {
            let (record, consumed) = DnsRecord::from_bytes(bytes, offset)?;
            authorities.push(record);
            offset = consumed;
        }

        let mut additional = Vec::with_capacity(arcount);
        for _ in 0..arcount {
            let (record, consumed) = DnsRecord::from_bytes(bytes, offset)?;
            additional.push(record);
            offset = consumed;
        }

        Ok(DnsPacket {
            header,
            questions,
            answers,
            authorities,
            additional,
        })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = vec![];

        self.header.write_to(&mut bytes);
        for question in &self.questions {
            question.write_to(&mut bytes);
        }

        // Answers, authorities, and additional records would go here

        bytes
    }

    pub fn extract_addresses(&self) -> Option<Vec<ResolvedAddress>> {
        let mut ips = Vec::new();
        for rec in &self.answers {
            match (rec.record_type, rec.resource_data.len()) {
                (1, 4) => {
                    let mut octets = [0u8; 4];
                    octets.copy_from_slice(&rec.resource_data[..4]);
                    let ip = IpAddr::V4(std::net::Ipv4Addr::from(octets));
                    // Parse TTL into Duration. Fallback to 0 if not enough length.
                    let ttl_secs = rec.time_to_live;
                    let ttl = std::time::Duration::from_secs(ttl_secs as u64);
                    ips.push(ResolvedAddress::new(ip, ttl));
                }
                (28, 16) => {
                    let mut octets = [0u8; 16];
                    octets.copy_from_slice(&rec.resource_data[..16]);
                    let ip = IpAddr::V6(std::net::Ipv6Addr::from(octets));
                    let ttl_secs = rec.time_to_live;
                    let ttl = std::time::Duration::from_secs(ttl_secs as u64);
                    ips.push(ResolvedAddress::new(ip, ttl));
                }
                _ => {}
            }
        }
        if ips.is_empty() { None } else { Some(ips) }
    }
}
