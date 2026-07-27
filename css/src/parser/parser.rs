use std::{unimplemented, vec};

use anyhow::{Ok, Result};
use html::dom::node::Node;
use utils::profile_scope;

use crate::{cssom::stylesheet::StyleSheet, tokenizer::tokenizer::Tokenizer};

pub struct Parser;

impl Parser {
    pub fn to_cssom(dom: Vec<Node>) -> Result<StyleSheet> {
        profile_scope!("To CSSOM");

        let mut tokenizer = Tokenizer::new(&dom);

        let tokens = tokenizer.tokenize();
        Ok(StyleSheet { rules: vec![] })
    }
}
