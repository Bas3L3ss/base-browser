use crate::{LengthContext, Unit};

#[derive(Debug, Clone, PartialEq)]
pub enum Length {
    Dimension { value: f32, unit: Unit },
    Percent(f32),
    Auto,
}

impl Length {
    /// Converts the Length to a f32 value
    ///
    /// - If self is `Dimension`, returns its converted value using context.
    /// - If self is `Percent`, interprets as a percent of context.percentage_base.
    /// - If self is `Auto`, returns the provided `auto` value or 0.0.
    pub fn to_f32(&self, of: f32, auto: Option<f32>, ctx: &LengthContext) -> f32 {
        match self {
            Length::Dimension { value, unit } => unit.to_px(*value, ctx).or(auto).unwrap_or(0.0),
            Length::Percent(p) => p * of,
            Length::Auto => auto.unwrap_or(0.0),
        }
    }
}
