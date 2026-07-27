use std::{println, vec};

use anyhow::{Ok, Result};
use utils::profile_scope;

use crate::{cssom::rule::Rule, tokenizer::tokenizer::Tokenizer};

pub struct Parser;

impl Parser {
    pub fn to_cssom(dom: &str) -> Result<Vec<Rule>> {
        profile_scope!("To CSSOM");

        let tokens = Tokenizer::from_string(dom);
        println!("{tokens:?}");
        Ok(vec![])
    }
}
