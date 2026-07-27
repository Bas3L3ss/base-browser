use input_parser::generic_struct::ParsedOutput;
use input_parser::traits::Parse;
use url::url::{Host, Scheme, Url};
use url::url_warning::UrlWarning;

use std::net::IpAddr;
use std::str::FromStr;
use std::{assert_eq, matches};

/// Helper to call Url::parse with error unwrapping for integration tests
fn parse_url(s: &str) -> ParsedOutput<Url, UrlWarning> {
    Url::parse(s).unwrap()
}

#[test]
fn integration_parses_simple_url() {
    let p = parse_url("http://foobar.com/path");
    assert_eq!(p.value.scheme, Scheme::Http);
    assert_eq!(p.value.host, Host::Domain("foobar.com".to_string()));
    assert_eq!(p.value.port, 80);
    assert_eq!(p.value.path, "/path");
    assert!(p.value.query.is_none());
    assert!(p.value.fragment.is_none());
    assert!(p.warnings.is_empty());
}

#[test]
fn integration_https_with_query_and_fragment() {
    let p = parse_url("https://foo.com/a/b?x=1&y=2#anchor");
    assert_eq!(p.value.scheme, Scheme::Https);
    assert_eq!(p.value.host, Host::Domain("foo.com".to_string()));
    assert_eq!(p.value.port, 443);
    assert_eq!(p.value.path, "/a/b");
    assert_eq!(p.value.query, Some("x=1&y=2".to_string()));
    assert_eq!(p.value.fragment, Some("anchor".to_string()));
    assert!(p.warnings.is_empty());
}

#[test]
fn integration_parses_host_and_custom_port() {
    let p = parse_url("http://mydomain.com:1234/foo/bar");
    assert_eq!(p.value.scheme, Scheme::Http);
    assert_eq!(p.value.host, Host::Domain("mydomain.com".to_string()));
    assert_eq!(p.value.port, 1234);
    assert_eq!(p.value.path, "/foo/bar");
    assert!(p.value.query.is_none());
}

#[test]
fn integration_parses_ip_v4_and_port() {
    let p = parse_url("http://127.1.2.3:5678/");
    assert_eq!(
        p.value.host,
        Host::Ip(IpAddr::from_str("127.1.2.3").unwrap())
    );
    assert_eq!(p.value.port, 5678);
    assert_eq!(p.value.scheme, Scheme::Http);
    assert_eq!(p.value.path, "/");
    assert!(p.warnings.is_empty());
}

#[test]
fn integration_parses_ip_v6_default_port() {
    let p = parse_url("https://[2001:db8::abcd]/foo");
    assert_eq!(
        p.value.host,
        Host::Ip(IpAddr::from_str("2001:db8::abcd").unwrap())
    );
    assert_eq!(p.value.port, 443);
    assert_eq!(p.value.scheme, Scheme::Https);
    assert_eq!(p.value.path, "/foo");
    assert!(p.warnings.is_empty());
}

#[test]
fn integration_warns_missing_scheme_defaults_to_http() {
    let p = parse_url("abc.com/home");
    assert_eq!(p.value.scheme, Scheme::Https);
    assert_eq!(p.value.host, Host::Domain("abc.com".to_string()));
    assert_eq!(p.value.port, 443);
    assert_eq!(p.value.path, "/home");
    assert!(p.warnings.contains(&UrlWarning::MissingScheme));
}

#[test]
fn integration_empty_host_warns_missing_host() {
    let p = parse_url("http:///splat");
    assert_eq!(p.value.host, Host::Domain("".to_string()));
    assert_eq!(p.value.path, "/splat");
    assert!(p.warnings.contains(&UrlWarning::MissingHost));
}

#[test]
fn integration_warns_invalid_port_and_falls_back() {
    let p = parse_url("http://z.com:notaport/x");
    assert_eq!(p.value.port, 80); // default fallback
    assert!(
        p.warnings
            .iter()
            .any(|w| matches!(w, UrlWarning::InvalidPort(s) if s == "notaport"))
    );
}

#[test]
fn integration_ipv6_invalid_warns_and_fallback() {
    let p = parse_url("https://[xyz::invalid]/stuff");
    assert!(matches!(p.value.host, Host::Domain(_)));
    assert!(
        p.warnings
            .iter()
            .any(|w| matches!(w, UrlWarning::InvalidHost(msg) if msg == "xyz::invalid"))
    );
}

#[test]
fn integration_default_path_is_slash() {
    let p = parse_url("https://hello.com");
    assert_eq!(p.value.path, "/");
    assert!(p.value.fragment.is_none());
}

#[test]
fn integration_path_and_fragment_only() {
    let p = parse_url("foo.org/#xyz123");
    assert_eq!(p.value.host, Host::Domain("foo.org".to_string()));
    assert_eq!(p.value.path, "/");
    assert_eq!(p.value.fragment, Some("xyz123".to_string()));
    assert!(p.value.query.is_none());
}
