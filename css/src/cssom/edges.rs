use crate::{Length, LengthContext};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Edges<T> {
    pub top: T,
    pub right: T,
    pub bottom: T,
    pub left: T,
}

impl<T> Edges<T> {
    pub fn new(top: T, right: T, bottom: T, left: T) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }
}

impl Edges<Length> {
    pub fn to_f32(&self, of: f32, auto: Option<f32>, ctx: &LengthContext) -> Edges<f32> {
        Edges {
            top: self.top.to_f32(of, auto, ctx),
            right: self.right.to_f32(of, auto, ctx),
            bottom: self.bottom.to_f32(of, auto, ctx),
            left: self.left.to_f32(of, auto, ctx),
        }
    }
}
