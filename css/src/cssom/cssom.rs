use crate::cssom::rule::Rule;

#[derive(Debug, Clone, PartialEq)]
pub struct CssOm {
    pub rules: Vec<Rule>,
}

impl CssOm {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    pub fn append_stylesheet(&mut self, rules: Vec<Rule>) {
        self.rules.extend(rules);
    }
}
