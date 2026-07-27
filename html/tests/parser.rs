use html::parser::parser::Parser;
use test_utils::fixture::html;

#[test]
fn parses_basic_inline_html() {
    let html = "<div>Hello</div>";

    let result = Parser::to_dom(html);
    assert!(result.is_ok(), "Expected parse success for valid HTML");
    let dom = result.unwrap();
    assert!(
        dom.warnings.is_empty(),
        "Expected no warnings for simple valid HTML"
    );
}

#[test]
fn parses_simple_html_fixture() {
    let html = &html("simple.html");
    let result = Parser::to_dom(html);
    assert!(result.is_ok(), "Expected parse success for fixture");
    let dom = result.unwrap();
    assert!(
        dom.warnings.is_empty(),
        "Fixture should not generate warnings"
    );
}

#[test]
fn parses_table_html_fixture() {
    let html = &html("table.html");
    let result = Parser::to_dom(html);
    assert!(result.is_ok(), "Table fixture should parse successfully");
    let dom = result.unwrap();
    assert!(
        dom.warnings.is_empty(),
        "Table fixture should be warning-free"
    );
}

#[test]
fn parses_nested_html_fixture() {
    let html = &html("nested.html");
    let result = Parser::to_dom(html);
    assert!(result.is_ok(), "Nested fixture should parse successfully");
    let dom = result.unwrap();
    assert!(
        dom.warnings.is_empty(),
        "Nested fixture should be warning-free"
    );
}

#[test]
fn reports_errors_for_malformed_html() {
    let html = &html("malformed.html");
    let result = Parser::to_dom(html);
    assert!(
        result.is_ok(),
        "Even malformed HTML should parse to a result"
    );
    let dom = result.unwrap();
    assert!(
        !dom.warnings.is_empty(),
        "Malformed fixture should produce parser warnings"
    );
}
