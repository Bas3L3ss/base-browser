use std::println;

use anyhow::{Ok, Result};

use crate::{
    parser::dom::Dom,
    tokenizer::{token::Token, tokenizer::Tokenizer},
};

pub fn parse(html: &str) -> Result<Dom> {
    let mut tokenizer = Tokenizer::new(html);
    let tokens: Vec<Token> = tokenizer.tokenize();
    println!("{tokens:?}");
    Ok(Dom {})
}
