#[derive(Debug, Clone, PartialEq)]
pub enum Color {
    Rgb(u8, u8, u8),
    Rgba(u8, u8, u8, f32),
    Hsl(f32, f32, f32),
    Hsla(f32, f32, f32, f32),
    Hex(String),
    Named(String),
}
