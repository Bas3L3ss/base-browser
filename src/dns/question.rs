use crate::dns::enums::{QueryClass, QueryType};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DnsQuestion {
    pub name: String,
    pub qtype: QueryType,
    pub qclass: QueryClass,
}

impl DnsQuestion {
    pub fn new(name: String, qtype: QueryType, qclass: QueryClass) -> DnsQuestion {
        DnsQuestion {
            name,
            qtype,
            qclass,
        }
    }
    pub fn write_to(&self, packet: &mut Vec<u8>) {
        for label in self.name.split('.') {
            packet.push(label.len() as u8);
            packet.extend(label.as_bytes());
        }
        packet.push(0);

        let qtype_num = match self.qtype {
            QueryType::A => 1,
            QueryType::NS => 2,
            QueryType::CNAME => 5,
            QueryType::SOA => 6,
            QueryType::PTR => 12,
            QueryType::MX => 15,
            QueryType::TXT => 16,
            QueryType::AAAA => 28,
            QueryType::ANY => 255,
        };
        packet.extend(&(qtype_num as u16).to_be_bytes());

        let qclass_num = match self.qclass {
            QueryClass::IN => 1,
            QueryClass::CH => 3,
            QueryClass::HS => 4,
            QueryClass::ANY => 255,
        };
        packet.extend(&(qclass_num as u16).to_be_bytes());
    }
}
