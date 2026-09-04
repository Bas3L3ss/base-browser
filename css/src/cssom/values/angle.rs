use std::f32::consts::PI;

#[derive(Debug, Clone, PartialEq)]
pub struct Angle {
    pub value: f32,
    pub unit: AngleUnit,
}

// Assume Angle implements to_degrees().
impl Angle {
    /// Converts self to degrees as f32, regardless of self.unit.
    pub fn to_degrees(&self) -> f32 {
        match self.unit {
            AngleUnit::Deg => self.value as f32,
            AngleUnit::Rad => self.value as f32 * (180.0 / PI),
            AngleUnit::Grad => self.value as f32 * 0.9,
            AngleUnit::Turn => self.value as f32 * 360.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AngleUnit {
    Deg,
    Grad,
    Rad,
    Turn,
}
