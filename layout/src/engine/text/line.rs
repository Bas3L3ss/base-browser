use std::vec;

use crate::text::inline_fragment::InlineFragment;

#[derive(Clone)]
pub struct Line {
    pub fragments: Vec<InlineFragment>,

    pub width: f32,
    pub height: f32,

    pub ascent: f32,
    pub descent: f32,
}

impl Line {
    pub fn new(height: f32, ascent: f32, descent: f32) -> Self {
        Self {
            fragments: vec![],
            width: 0.0,
            height,
            ascent,
            descent,
        }
    }
}
