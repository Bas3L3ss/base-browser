use std::vec;

use crate::dns::{header::Header, question::DnsQuestion};

pub struct DnsPacket {
    header: Header,
    questions: Vec<DnsQuestion>,
    // later add answers, authorities, and additionals
}

impl DnsPacket {
    pub fn new(header: Header, questions: Vec<DnsQuestion>) -> Self {
        DnsPacket { header, questions }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = vec![];

        self.header.write_to(&mut bytes);
        for question in &self.questions {
            question.write_to(&mut bytes);
        }

        bytes
    }
}
