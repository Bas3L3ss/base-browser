use std::todo;

use crate::face::{FontFace, FontStyle, FontWeight};

pub struct FontDatabase {
    faces: Vec<FontFace>,
}

impl FontDatabase {
    pub fn register(&mut self, face: FontFace) {
        todo!()
    }

    pub fn lookup(&self, family: &str, weight: FontWeight, style: FontStyle) -> Option<&FontFace> {
        todo!()
    }
}
