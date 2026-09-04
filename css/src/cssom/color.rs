use crate::{ComponentValue, Function};

#[derive(Debug, Clone, PartialEq)]
pub enum Color {
    Rgb(u8, u8, u8),
    Rgba(u8, u8, u8, f32),
    Hsl(f32, f32, f32),
    Hsla(f32, f32, f32, f32),
    Hex(String),
    Named(String),
}

impl Color {
    // Parse for rgb, rgba, hsl, hsla from a function name and argument vector
    pub fn parse_function(function: &Function) -> Option<Color> {
        match function.name.to_ascii_lowercase().as_str() {
            "rgb" => {
                if function.args.len() == 3 {
                    let r = Self::parse_color_component(&function.args[0])?;
                    let g = Self::parse_color_component(&function.args[1])?;
                    let b = Self::parse_color_component(&function.args[2])?;
                    Some(Self::Rgb(r, g, b))
                } else {
                    None
                }
            }
            "rgba" => {
                if function.args.len() == 4 {
                    let r = Self::parse_color_component(&function.args[0])?;
                    let g = Self::parse_color_component(&function.args[1])?;
                    let b = Self::parse_color_component(&function.args[2])?;
                    let a = Self::parse_alpha_component(&function.args[3])?;
                    Some(Self::Rgba(r, g, b, a))
                } else {
                    None
                }
            }
            "hsl" => {
                if function.args.len() == 3 {
                    let h = Self::parse_angle(&function.args[0])?;
                    let s = Self::parse_percentage(&function.args[1])?;
                    let l = Self::parse_percentage(&function.args[2])?;
                    Some(Self::Hsl(h, s, l))
                } else {
                    None
                }
            }
            "hsla" => {
                if function.args.len() == 4 {
                    let h = Self::parse_angle(&function.args[0])?;
                    let s = Self::parse_percentage(&function.args[1])?;
                    let l = Self::parse_percentage(&function.args[2])?;
                    let a = Self::parse_alpha_component(&function.args[3])?;
                    Some(Self::Hsla(h, s, l, a))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn parse_color_component(val: &ComponentValue) -> Option<u8> {
        // Try as f32 then map into u8 range if possible
        if let Some(num) = val.as_percentage() {
            if num >= 0.0 && num <= 100.0 {
                return Some((num / 100.0 * 255.0).round() as u8);
            }
        } else if let Some(n) = match val {
            ComponentValue::Number(v) => Some(*v),
            _ => None,
        } {
            if n >= 0.0 && n <= 255.0 {
                return Some(n.round() as u8);
            }
        }
        None
    }

    fn parse_alpha_component(val: &ComponentValue) -> Option<f32> {
        if let Some(percent) = val.as_percentage() {
            if percent >= 0.0 && percent <= 100.0 {
                return Some(percent / 100.0);
            }
        }
        // Try as number, which could be already in [0,1] or [0,100]
        match val {
            ComponentValue::Number(num) if *num >= 0.0 && *num <= 1.0 => Some(*num),
            ComponentValue::Number(num) if *num >= 0.0 && *num <= 100.0 => Some(*num / 100.0),
            _ => None,
        }
    }

    fn parse_percentage(val: &ComponentValue) -> Option<f32> {
        val.as_percentage().or_else(|| match val {
            ComponentValue::Number(num) => Some(*num),
            _ => None,
        })
    }

    fn parse_angle(val: &ComponentValue) -> Option<f32> {
        match val {
            ComponentValue::Number(num) => Some(*num),
            _ => None,
        }
    }
}
