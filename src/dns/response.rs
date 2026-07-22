use std::{net::IpAddr, time::Duration};

#[derive(Debug)]
pub struct ResolvedAddress {
    pub ip: IpAddr,
    pub ttl: Duration,
}

impl ResolvedAddress {
    pub fn new(ip: IpAddr, ttl: Duration) -> Self {
        Self { ip, ttl }
    }
}

#[derive(Debug)]
pub struct LookupResult {
    pub addresses: Vec<ResolvedAddress>,
}

impl LookupResult {
    pub fn new(addresses: Vec<ResolvedAddress>) -> Self {
        Self { addresses }
    }
}
