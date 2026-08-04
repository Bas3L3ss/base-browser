use crate::Unit;

#[derive(Debug, Clone, PartialEq)]
pub enum Length {
    Dimension { value: f32, unit: Unit },
    Percent(f32),
    Auto,
}
