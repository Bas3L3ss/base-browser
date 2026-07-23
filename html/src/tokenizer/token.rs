use crate::tokenizer::{attributes::Attributes, html_tags::HtmlTags};
use std::fmt;

#[derive(Clone, PartialEq, Eq)]
pub enum Token {
    Start(HtmlTags, Attributes),
    SelfClose(HtmlTags, Attributes),
    End(HtmlTags),
    Content(String),
    Comment(String),
    Doctype(String),
    EOF,
}

impl fmt::Debug for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Start(tag, _) => write!(f, "Start({:?}, ...)", tag),
            Token::SelfClose(tag, _) => write!(f, "SelfClose({:?}, ...)", tag),
            Token::End(tag) => write!(f, "End({:?})", tag),
            Token::Content(s) => write!(f, "Content({:?})", s),
            Token::Comment(s) => write!(f, "Comment({:?})", s),
            Token::Doctype(s) => write!(f, "Doctype({:?})", s),
            Token::EOF => write!(f, "EOF"),
        }
    }
}
