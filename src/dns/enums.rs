// ===================
// QUERY CLASS
// ===================

/// QueryClass values for the DNS QCLASS field, as specified in RFC 1035.
/// Used to indicate the class of a DNS query, most commonly IN (the internet).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryClass {
    /// the Internet
    IN = 1,
    /// the CHAOS class
    CH = 3,
    /// Hesiod
    HS = 4,
    /// Any class (wildcard, RFC 1035 3.2.5)
    ANY = 255,
}

impl Default for QueryClass {
    fn default() -> Self {
        QueryClass::IN
    }
}

impl QueryClass {
    /// Convert to wire-format `u16` value.
    #[inline]
    pub fn to_u16(self) -> u16 {
        self as u16
    }

    /// Convert from a wire-format `u16` value.
    #[inline]
    pub fn from_u16(n: u16) -> Self {
        match n {
            1 => QueryClass::IN,
            3 => QueryClass::CH,
            4 => QueryClass::HS,
            255 => QueryClass::ANY,
            _ => QueryClass::IN, // fallback: treat as IN
        }
    }
}

impl From<u16> for QueryClass {
    fn from(n: u16) -> Self {
        QueryClass::from_u16(n)
    }
}

impl From<QueryClass> for u16 {
    fn from(class: QueryClass) -> u16 {
        class.to_u16()
    }
}

// ----------------------------------------------------------------------

// ===================
// QUERY TYPE
// ===================

/// QueryType values for the DNS QTYPE and TYPE fields, as per RFC 1035 and its updates.
/// Used to indicate what record type is being queried or represented.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryType {
    /// A Record (IPv4 address)
    A = 1,
    /// Authoritative Name Server
    NS = 2,
    /// Canonical Name
    CNAME = 5,
    /// Start of Authority
    SOA = 6,
    /// Domain Name Pointer (reverse records)
    PTR = 12,
    /// Mail Exchange
    MX = 15,
    /// Arbitrary (text) strings
    TXT = 16,
    /// AAAA Record (IPv6 address)
    AAAA = 28,
    /// ANY type (wildcard, RFC 1035 3.2.3)
    ANY = 255,
}

impl Default for QueryType {
    fn default() -> Self {
        QueryType::A
    }
}

impl QueryType {
    /// Convert to wire-format `u16` value.
    #[inline]
    pub fn to_u16(self) -> u16 {
        self as u16
    }

    /// Convert from a wire-format `u16` value.
    #[inline]
    pub fn from_u16(n: u16) -> Self {
        match n {
            1 => QueryType::A,
            2 => QueryType::NS,
            5 => QueryType::CNAME,
            6 => QueryType::SOA,
            12 => QueryType::PTR,
            15 => QueryType::MX,
            16 => QueryType::TXT,
            28 => QueryType::AAAA,
            255 => QueryType::ANY,
            _ => QueryType::A, // fallback: treat as A
        }
    }
}

impl From<u16> for QueryType {
    fn from(n: u16) -> Self {
        QueryType::from_u16(n)
    }
}

impl From<QueryType> for u16 {
    fn from(qtype: QueryType) -> u16 {
        qtype.to_u16()
    }
}

// ----------------------------------------------------------------------

// ===================
// OPCODE
// ===================

/// DNS Opcode, as per RFC 1035 section 4.1.1.
/// Operation code that specifies the kind of query (standard, status, update, etc).
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Opcode {
    /// Standard query (QUERY)
    Query = 0,
    /// Inverse query (obsolete)
    IQuery = 1,
    /// Server status request (RFC 1035 4.1.1)
    Status = 2,
    /// Notify (RFC 1996)
    Notify = 4,
    /// Dynamic update (RFC 2136)
    Update = 5,
    /// Any other value (unknown or reserved)
    Unknown(u8),
}

impl From<u8> for Opcode {
    fn from(value: u8) -> Self {
        match value {
            0 => Opcode::Query,
            1 => Opcode::IQuery,
            2 => Opcode::Status,
            4 => Opcode::Notify,
            5 => Opcode::Update,
            v => Opcode::Unknown(v),
        }
    }
}

impl From<Opcode> for u8 {
    fn from(op: Opcode) -> Self {
        match op {
            Opcode::Query => 0,
            Opcode::IQuery => 1,
            Opcode::Status => 2,
            Opcode::Notify => 4,
            Opcode::Update => 5,
            Opcode::Unknown(v) => v,
        }
    }
}

// ----------------------------------------------------------------------

// ===================
// RCODE
// ===================

/// DNS Rcode, as per RFC 1035 and later extensions (RFC 2136, etc).
/// Response codes indicate errors and conditions for queries/responses.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rcode {
    /// No Error
    NoError = 0,
    /// Format Error - The name server was unable to interpret the query.
    FormErr = 1,
    /// Server Failure - The name server was unable to process this query due to a problem with the name server.
    ServFail = 2,
    /// Non-Existent Domain - The domain name referenced in the query does not exist.
    NXDomain = 3,
    /// Not Implemented - The name server does not support the requested kind of query.
    NotImp = 4,
    /// Refused - The name server refuses to perform the specified operation.
    Refused = 5,
    /// Name Exists when it should not (RFC 2136)
    YXDomain = 6,
    /// RR Set Exists when it should not (RFC 2136)
    YXRRSet = 7,
    /// RR Set that should exist does not (RFC 2136)
    NXRRSet = 8,
    /// Server Not Authoritative for zone (RFC 2136)
    NotAuth = 9,
    /// Name not contained in zone (RFC 2136)
    NotZone = 10,
    /// Any other value (unknown or reserved)
    Unknown(u8),
}

impl From<u8> for Rcode {
    fn from(value: u8) -> Self {
        match value {
            0 => Rcode::NoError,
            1 => Rcode::FormErr,
            2 => Rcode::ServFail,
            3 => Rcode::NXDomain,
            4 => Rcode::NotImp,
            5 => Rcode::Refused,
            6 => Rcode::YXDomain,
            7 => Rcode::YXRRSet,
            8 => Rcode::NXRRSet,
            9 => Rcode::NotAuth,
            10 => Rcode::NotZone,
            v => Rcode::Unknown(v),
        }
    }
}

impl From<Rcode> for u8 {
    fn from(rcode: Rcode) -> Self {
        match rcode {
            Rcode::NoError => 0,
            Rcode::FormErr => 1,
            Rcode::ServFail => 2,
            Rcode::NXDomain => 3,
            Rcode::NotImp => 4,
            Rcode::Refused => 5,
            Rcode::YXDomain => 6,
            Rcode::YXRRSet => 7,
            Rcode::NXRRSet => 8,
            Rcode::NotAuth => 9,
            Rcode::NotZone => 10,
            Rcode::Unknown(v) => v,
        }
    }
}
