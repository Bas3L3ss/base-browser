use crate::CssValue;

pub struct WhiteSpaceBehavior {
    pub collapse_spaces: bool,
    pub preserve_newlines: bool,
    pub allow_wrap: bool,
    // In the future, if special behavior for break-spaces is needed, add another field
}

impl Default for WhiteSpaceBehavior {
    fn default() -> Self {
        // Default to "normal"
        WhiteSpaceBehavior {
            collapse_spaces: true,
            preserve_newlines: false,
            allow_wrap: true,
        }
    }
}

impl WhiteSpaceBehavior {
    pub fn from_css(value: &CssValue) -> Self {
        if let Some(ident) = value.components.first().and_then(|c| c.as_ident()) {
            match ident {
                "normal" => WhiteSpaceBehavior {
                    collapse_spaces: true,
                    preserve_newlines: false,
                    allow_wrap: true,
                },
                "nowrap" => WhiteSpaceBehavior {
                    collapse_spaces: true,
                    preserve_newlines: false,
                    allow_wrap: false,
                },
                "pre" => WhiteSpaceBehavior {
                    collapse_spaces: false,
                    preserve_newlines: true,
                    allow_wrap: false,
                },
                "pre-wrap" => WhiteSpaceBehavior {
                    collapse_spaces: false,
                    preserve_newlines: true,
                    allow_wrap: true,
                },
                "pre-line" => WhiteSpaceBehavior {
                    collapse_spaces: true,
                    preserve_newlines: true,
                    allow_wrap: true,
                },
                "break-spaces" => WhiteSpaceBehavior {
                    collapse_spaces: false,
                    preserve_newlines: true,
                    allow_wrap: true, // Note: Special space-breaking behavior not yet represented
                },
                _ => WhiteSpaceBehavior::default(),
            }
        } else {
            WhiteSpaceBehavior::default()
        }
    }
}
