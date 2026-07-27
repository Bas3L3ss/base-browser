use std::format;

pub fn html(name: &str) -> String {
    std::fs::read_to_string(format!("../test_fixtures/html/{name}")).unwrap()
}
