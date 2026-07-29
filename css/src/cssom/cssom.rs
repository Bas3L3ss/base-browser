use crate::cssom::rule::CssRule;

#[derive(Debug, Clone, PartialEq)]
pub struct CssOm {
    pub rules: Vec<CssRule>,
}

impl CssOm {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    pub fn append_stylesheet(&mut self, rules: Vec<CssRule>) {
        self.rules.extend(rules);
    }
}
