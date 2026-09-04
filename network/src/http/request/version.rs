use core::fmt;
use std::write;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HttpVersion {
    Http10,
    Http11,
    // Http20,
    // Http30,
}

impl fmt::Display for HttpVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            HttpVersion::Http10 => "HTTP/1.0",
            HttpVersion::Http11 => "HTTP/1.1",
            // HttpVersion::Http20 => "HTTP/2.0",
            // HttpVersion::Http30 => "HTTP/3.0",
        };
        write!(f, "{s}")
    }
}
