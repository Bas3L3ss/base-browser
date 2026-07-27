use std::format;

pub fn html(name: &str) -> String {
    std::fs::read_to_string(format!("../test-fixtures/html/{name}")).unwrap()
}
