#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryClass {
    IN,  // the Internet
    CH,  // the CHAOS class
    HS,  // Hesiod
    ANY, // Any class
}

impl Default for QueryClass {
    fn default() -> Self {
        QueryClass::IN
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryType {
    A,     // IPv4 address
    NS,    // Authoritative Name Server
    CNAME, // Canonical Name
    SOA,   // Start of Authority
    PTR,   // Domain name pointer
    MX,    // Mail exchange
    TXT,   // Text strings
    AAAA,  // IPv6 address
    ANY,   // Any type
}

impl Default for QueryType {
    fn default() -> Self {
        QueryType::A
    }
}
