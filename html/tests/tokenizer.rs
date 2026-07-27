use html::tokenizer::tokenizer::Tokenizer;
use utils::test_fixture::html;

#[test]
fn tokenizes_basic_inline_html() {
    let html = "<div>Hello</div>";

    let mut tokenizer = Tokenizer::new(html);
    let tokens = tokenizer.tokenize();
    assert!(!tokens.is_empty(), "Expected tokens from valid HTML");
}

#[test]
fn tokenizes_simple_html_fixture() {
    let html = &html("simple.html");
    let mut tokenizer = Tokenizer::new(html);
    let tokens = tokenizer.tokenize();
    assert!(
        !tokens.is_empty(),
        "Expected tokens from simple.html fixture"
    );
}

#[test]
fn tokenizes_table_html_fixture() {
    let html = &html("table.html");
    let mut tokenizer = Tokenizer::new(html);
    let tokens = tokenizer.tokenize();
    assert!(
        !tokens.is_empty(),
        "Expected tokens from table.html fixture"
    );
}

#[test]
fn tokenizes_nested_html_fixture() {
    let html = &html("nested.html");
    let mut tokenizer = Tokenizer::new(html);
    let tokens = tokenizer.tokenize();
    assert!(
        !tokens.is_empty(),
        "Expected tokens from nested.html fixture"
    );
}

// The Tokenizer does not support error reporting, so testing for tokenizer.errors() is not valid.
// If error interface is added in the future, update this test accordingly.
#[test]
fn tokenizes_malformed_html() {
    let html = &html("malformed.html");
    let mut tokenizer = Tokenizer::new(html);
    let tokens = tokenizer.tokenize();
    assert!(
        !tokens.is_empty(),
        "Tokenizer should still produce tokens for malformed HTML"
    );
}
