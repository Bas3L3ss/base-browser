pub struct MediaQuery {
    /// not / only
    pub modifier: Option<MediaModifier>,

    /// screen, print, all
    pub media_type: Option<MediaType>,

    /// and/or/not conditions
    pub condition: Option<MediaCondition>,
}

pub enum MediaType {
    All,
    Screen,
    Print,
    Speech,
}

pub enum MediaModifier {
    Not,
    Only,
}

pub enum MediaCondition {
    Feature(MediaFeature),

    And(Box<MediaCondition>, Box<MediaCondition>),

    Or(Box<MediaCondition>, Box<MediaCondition>),

    Not(Box<MediaCondition>),
}

pub enum MediaFeature {
    // Width(Comparison),
    // Height(Comparison),

    // DeviceWidth(Comparison),
    // DeviceHeight(Comparison),

    // Orientation(Orientation),

    // Resolution(Resolution),

    // ColorScheme(ColorScheme),

    // Hover(Hover),

    // Pointer(Pointer),

    // AspectRatio(Ratio),

    // Custom { name: String, value: TokenList },
}
