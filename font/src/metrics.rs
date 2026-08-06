#[derive(Debug, Clone, PartialEq, Default)]
pub struct FontMetrics {
    pub units_per_em: u16,

    pub ascent: f32,

    pub descent: f32,

    pub line_gap: f32,
}
