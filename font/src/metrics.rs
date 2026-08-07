#[derive(Debug, Clone, PartialEq, Default)]
pub struct FontMetrics {
    pub units_per_em: u16,

    pub ascent: i16,
    pub descent: i16,
    pub line_gap: i16,
}
