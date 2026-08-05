use crate::rule::{at_rule::AtRule, style_rule::StyleRule};
#[derive(Debug, Clone, PartialEq)]
pub enum CssRule {
    Style(StyleRule),
    At(AtRule),
}
