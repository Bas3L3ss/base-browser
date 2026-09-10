use crate::{ ComponentValue, Function };

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
    pub fn to_rgba(&self) -> (u8, u8, u8, f32) {
        match self {
            Color::Rgb(r, g, b) => (*r, *g, *b, 1.0),
            Color::Rgba(r, g, b, a) => (*r, *g, *b, *a),
            Color::Hsl(h, s, l) => {
                let (r, g, b) = Self::hsl_to_rgb(*h, *s, *l);
                (r, g, b, 1.0)
            }
            Color::Hsla(h, s, l, a) => {
                let (r, g, b) = Self::hsl_to_rgb(*h, *s, *l);
                (r, g, b, *a)
            }
            Color::Hex(code) => {
                if let Some((r, g, b, a)) = Self::parse_hex(code) {
                    (r, g, b, a)
                } else {
                    (0, 0, 0, 1.0)
                }
            }
            Color::Named(name) => {
                if let Some((r, g, b, a)) = Self::named_color_to_rgba(name) {
                    (r, g, b, a)
                } else {
                    (0, 0, 0, 1.0)
                }
            }
        }
    }

    fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (u8, u8, u8) {
        // h in [0,360), s/l in [0,100]
        let (h, s, l) = (h % 360.0, s.max(0.0).min(100.0) / 100.0, l.max(0.0).min(100.0) / 100.0);

        let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
        let h_ = h / 60.0;
        let x = c * (1.0 - ((h_ % 2.0) - 1.0).abs());
        let (r1, g1, b1) = match h_ as u32 {
            0 => (c, x, 0.0),
            1 => (x, c, 0.0),
            2 => (0.0, c, x),
            3 => (0.0, x, c),
            4 => (x, 0.0, c),
            5 => (c, 0.0, x),
            _ => (0.0, 0.0, 0.0),
        };
        let m = l - c / 2.0;
        let (r, g, b) = ((r1 + m) * 255.0, (g1 + m) * 255.0, (b1 + m) * 255.0);

        (r.round() as u8, g.round() as u8, b.round() as u8)
    }

    fn parse_hex(s: &str) -> Option<(u8, u8, u8, f32)> {
        panic!("{s}");
        let hex = s.trim_start_matches('#');
        match hex.len() {
            3 => {
                let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?;
                let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?;
                let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?;
                Some((r, g, b, 1.0))
            }
            4 => {
                let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?;
                let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?;
                let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?;
                let a = u8::from_str_radix(&hex[3..4].repeat(2), 16).ok()?;
                Some((r, g, b, (a as f32) / 255.0))
            }
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                Some((r, g, b, 1.0))
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
                Some((r, g, b, (a as f32) / 255.0))
            }
            _ => None,
        }
    }

    fn named_color_to_rgba(name: &str) -> Option<(u8, u8, u8, f32)> {
        // TODO: have a file that actually store these
        match name.to_ascii_lowercase().as_str() {
            "black" => Some((0, 0, 0, 1.0)),
            "white" => Some((255, 255, 255, 1.0)),
            "red" => Some((255, 0, 0, 1.0)),
            "green" => Some((0, 128, 0, 1.0)),
            "blue" => Some((0, 0, 255, 1.0)),
            "yellow" => Some((255, 255, 0, 1.0)),
            "cyan" => Some((0, 255, 255, 1.0)),
            "aqua" => Some((0, 255, 255, 1.0)),
            "magenta" => Some((255, 0, 255, 1.0)),
            "fuchsia" => Some((255, 0, 255, 1.0)),
            "gray" => Some((128, 128, 128, 1.0)),
            "grey" => Some((128, 128, 128, 1.0)),
            "maroon" => Some((128, 0, 0, 1.0)),
            "olive" => Some((128, 128, 0, 1.0)),
            "purple" => Some((128, 0, 128, 1.0)),
            "teal" => Some((0, 128, 128, 1.0)),
            "navy" => Some((0, 0, 128, 1.0)),
            "silver" => Some((192, 192, 192, 1.0)),
            "lime" => Some((0, 255, 0, 1.0)),
            "orange" => Some((255, 165, 0, 1.0)),
            "aliceblue" => Some((240, 248, 255, 1.0)),
            "antiquewhite" => Some((250, 235, 215, 1.0)),
            "azure" => Some((240, 255, 255, 1.0)),
            "beige" => Some((245, 245, 220, 1.0)),
            "bisque" => Some((255, 228, 196, 1.0)),
            "blanchedalmond" => Some((255, 235, 205, 1.0)),
            "blueviolet" => Some((138, 43, 226, 1.0)),
            "brown" => Some((165, 42, 42, 1.0)),
            "burlywood" => Some((222, 184, 135, 1.0)),
            "cadetblue" => Some((95, 158, 160, 1.0)),
            "chocolate" => Some((210, 105, 30, 1.0)),
            "coral" => Some((255, 127, 80, 1.0)),
            "cornflowerblue" => Some((100, 149, 237, 1.0)),
            "cornsilk" => Some((255, 248, 220, 1.0)),
            "crimson" => Some((220, 20, 60, 1.0)),
            "darkblue" => Some((0, 0, 139, 1.0)),
            "darkcyan" => Some((0, 139, 139, 1.0)),
            "darkgoldenrod" => Some((184, 134, 11, 1.0)),
            "darkgray" => Some((169, 169, 169, 1.0)),
            "darkgrey" => Some((169, 169, 169, 1.0)),
            "darkgreen" => Some((0, 100, 0, 1.0)),
            "darkkhaki" => Some((189, 183, 107, 1.0)),
            "darkmagenta" => Some((139, 0, 139, 1.0)),
            "darkolivegreen" => Some((85, 107, 47, 1.0)),
            "darkorange" => Some((255, 140, 0, 1.0)),
            "darkorchid" => Some((153, 50, 204, 1.0)),
            "darkred" => Some((139, 0, 0, 1.0)),
            "darksalmon" => Some((233, 150, 122, 1.0)),
            "darkseagreen" => Some((143, 188, 143, 1.0)),
            "darkslateblue" => Some((72, 61, 139, 1.0)),
            "darkslategray" => Some((47, 79, 79, 1.0)),
            "darkslategrey" => Some((47, 79, 79, 1.0)),
            "darkturquoise" => Some((0, 206, 209, 1.0)),
            "darkviolet" => Some((148, 0, 211, 1.0)),
            "deeppink" => Some((255, 20, 147, 1.0)),
            "deepskyblue" => Some((0, 191, 255, 1.0)),
            "dimgray" => Some((105, 105, 105, 1.0)),
            "dimgrey" => Some((105, 105, 105, 1.0)),
            "dodgerblue" => Some((30, 144, 255, 1.0)),
            "firebrick" => Some((178, 34, 34, 1.0)),
            "floralwhite" => Some((255, 250, 240, 1.0)),
            "forestgreen" => Some((34, 139, 34, 1.0)),
            "gainsboro" => Some((220, 220, 220, 1.0)),
            "ghostwhite" => Some((248, 248, 255, 1.0)),
            "gold" => Some((255, 215, 0, 1.0)),
            "goldenrod" => Some((218, 165, 32, 1.0)),
            "greenyellow" => Some((173, 255, 47, 1.0)),
            "honeydew" => Some((240, 255, 240, 1.0)),
            "hotpink" => Some((255, 105, 180, 1.0)),
            "indianred" => Some((205, 92, 92, 1.0)),
            "indigo" => Some((75, 0, 130, 1.0)),
            "ivory" => Some((255, 255, 240, 1.0)),
            "khaki" => Some((240, 230, 140, 1.0)),
            "lavender" => Some((230, 230, 250, 1.0)),
            "lavenderblush" => Some((255, 240, 245, 1.0)),
            "lawngreen" => Some((124, 252, 0, 1.0)),
            "lemonchiffon" => Some((255, 250, 205, 1.0)),
            "lightblue" => Some((173, 216, 230, 1.0)),
            "lightcoral" => Some((240, 128, 128, 1.0)),
            "lightcyan" => Some((224, 255, 255, 1.0)),
            "lightgoldenrodyellow" => Some((250, 250, 210, 1.0)),
            "lightgray" => Some((211, 211, 211, 1.0)),
            "lightgrey" => Some((211, 211, 211, 1.0)),
            "lightgreen" => Some((144, 238, 144, 1.0)),
            "lightpink" => Some((255, 182, 193, 1.0)),
            "lightsalmon" => Some((255, 160, 122, 1.0)),
            "lightseagreen" => Some((32, 178, 170, 1.0)),
            "lightskyblue" => Some((135, 206, 250, 1.0)),
            "lightslategray" => Some((119, 136, 153, 1.0)),
            "lightslategrey" => Some((119, 136, 153, 1.0)),
            "lightsteelblue" => Some((176, 196, 222, 1.0)),
            "lightyellow" => Some((255, 255, 224, 1.0)),
            "mediumaquamarine" => Some((102, 205, 170, 1.0)),
            "mediumblue" => Some((0, 0, 205, 1.0)),
            "mediumorchid" => Some((186, 85, 211, 1.0)),
            "mediumpurple" => Some((147, 112, 219, 1.0)),
            "mediumseagreen" => Some((60, 179, 113, 1.0)),
            "mediumslateblue" => Some((123, 104, 238, 1.0)),
            "mediumspringgreen" => Some((0, 250, 154, 1.0)),
            "mediumturquoise" => Some((72, 209, 204, 1.0)),
            "mediumvioletred" => Some((199, 21, 133, 1.0)),
            "midnightblue" => Some((25, 25, 112, 1.0)),
            "mintcream" => Some((245, 255, 250, 1.0)),
            "mistyrose" => Some((255, 228, 225, 1.0)),
            "moccasin" => Some((255, 228, 181, 1.0)),
            "navajowhite" => Some((255, 222, 173, 1.0)),
            "oldlace" => Some((253, 245, 230, 1.0)),
            "olivedrab" => Some((107, 142, 35, 1.0)),
            "orangered" => Some((255, 69, 0, 1.0)),
            "orchid" => Some((218, 112, 214, 1.0)),
            "palegoldenrod" => Some((238, 232, 170, 1.0)),
            "palegreen" => Some((152, 251, 152, 1.0)),
            "paleturquoise" => Some((175, 238, 238, 1.0)),
            "palevioletred" => Some((219, 112, 147, 1.0)),
            "papayawhip" => Some((255, 239, 213, 1.0)),
            "peachpuff" => Some((255, 218, 185, 1.0)),
            "peru" => Some((205, 133, 63, 1.0)),
            "pink" => Some((255, 192, 203, 1.0)),
            "plum" => Some((221, 160, 221, 1.0)),
            "powderblue" => Some((176, 224, 230, 1.0)),
            "rosybrown" => Some((188, 143, 143, 1.0)),
            "royalblue" => Some((65, 105, 225, 1.0)),
            "saddlebrown" => Some((139, 69, 19, 1.0)),
            "salmon" => Some((250, 128, 114, 1.0)),
            "sandybrown" => Some((244, 164, 96, 1.0)),
            "seagreen" => Some((46, 139, 87, 1.0)),
            "seashell" => Some((255, 245, 238, 1.0)),
            "sienna" => Some((160, 82, 45, 1.0)),
            "skyblue" => Some((135, 206, 235, 1.0)),
            "slateblue" => Some((106, 90, 205, 1.0)),
            "slategray" => Some((112, 128, 144, 1.0)),
            "slategrey" => Some((112, 128, 144, 1.0)),
            "snow" => Some((255, 250, 250, 1.0)),
            "springgreen" => Some((0, 255, 127, 1.0)),
            "steelblue" => Some((70, 130, 180, 1.0)),
            "tan" => Some((210, 180, 140, 1.0)),
            "thistle" => Some((216, 191, 216, 1.0)),
            "tomato" => Some((255, 99, 71, 1.0)),
            "turquoise" => Some((64, 224, 208, 1.0)),
            "violet" => Some((238, 130, 238, 1.0)),
            "wheat" => Some((245, 222, 179, 1.0)),
            "whitesmoke" => Some((245, 245, 245, 1.0)),
            "yellowgreen" => Some((154, 205, 50, 1.0)),
            // Add more as needed
            _ => None,
        }
    }

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
        if let Some(num) = val.as_percentage() {
            if num >= 0.0 && num <= 100.0 {
                return Some(((num / 100.0) * 255.0).round() as u8);
            }
        } else if
            let Some(n) = (match val {
                ComponentValue::Number(v) => Some(*v),
                _ => None,
            })
        {
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
        val.as_percentage().or_else(|| {
            match val {
                ComponentValue::Number(num) => Some(*num),
                _ => None,
            }
        })
    }

    fn parse_angle(val: &ComponentValue) -> Option<f32> {
        match val {
            ComponentValue::Number(num) => Some(*num),
            _ => None,
        }
    }
}
