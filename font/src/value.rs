#[derive(Debug, Clone, Copy)]
pub struct FontWeight(pub u16);

#[derive(Debug, Clone, Copy)]
pub struct FontStretch(pub f32);

#[derive(Debug, Clone, Copy)]
pub enum FontStyle {
    Normal,
    Italic,
    Oblique(f32),
}

impl FontStyle {
    pub fn from_name_angle(name: &str, angle: f32) -> Self {
        match name.to_ascii_lowercase().as_str() {
            "normal" => FontStyle::Normal,
            "italic" => FontStyle::Italic,
            "oblique" => FontStyle::Oblique(angle),
            _ => FontStyle::Normal, // fallback to normal if unknown
        }
    }
}
