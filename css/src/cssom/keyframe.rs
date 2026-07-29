use crate::cssom::declaration::Declaration;

#[derive(Debug, Clone, PartialEq)]
pub struct Keyframe {
    pub selectors: Vec<KeyframeSelector>,
    pub declarations: Vec<Declaration>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum KeyframeSelector {
    From,
    To,
    Percentage(f32),
}
