use crate::FontRequest;
use css::{FontFaceRule, ObliqueStyle};
use std::cmp::Ordering;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum StyleCategory {
    Exact,
    ItalicOblique,
    NormalFallback,
    NoMatch,
}

#[derive(Debug, PartialEq)]
pub(crate) struct StyleMatch {
    pub category: StyleCategory,
    pub distance: f32,
}

impl Eq for StyleMatch {}
impl PartialOrd for StyleMatch {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for StyleMatch {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.category.cmp(&other.category) {
            Ordering::Equal => self
                .distance
                .partial_cmp(&other.distance)
                .unwrap_or(Ordering::Equal)
                .reverse(),
            ord => ord.reverse(), // Lower enum value is preferred
        }
    }
}

#[derive(Debug, PartialEq)]
pub(crate) struct WeightMatch {
    pub distance: f32,
}
impl Eq for WeightMatch {}
impl PartialOrd for WeightMatch {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for WeightMatch {
    fn cmp(&self, other: &Self) -> Ordering {
        self.distance
            .partial_cmp(&other.distance)
            .unwrap_or(Ordering::Equal)
            .reverse()
    }
}

#[derive(Debug, PartialEq)]
pub(crate) struct StretchMatch {
    pub distance: f32,
}
impl Eq for StretchMatch {}
impl PartialOrd for StretchMatch {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for StretchMatch {
    fn cmp(&self, other: &Self) -> Ordering {
        self.distance
            .partial_cmp(&other.distance)
            .unwrap_or(Ordering::Equal)
            .reverse()
    }
}

pub(crate) trait FontCandidate {
    fn compare_style(&self, request: &FontRequest) -> StyleMatch;
    fn compare_weight(&self, request: &FontRequest) -> WeightMatch;
    fn compare_stretch(&self, request: &FontRequest) -> StretchMatch;
}

pub(crate) struct FontMatcher;

impl FontMatcher {
    fn compare_stage<T: Ord>(a: T, b: T) -> Option<Ordering> {
        match a.cmp(&b) {
            Ordering::Equal => None,
            ord => Some(ord),
        }
    }

    /// Compares two candidates with a given request, returning Ordering (CSS font matching logic).
    fn compare_face<T: FontCandidate>(a: &T, b: &T, request: &FontRequest) -> Ordering {
        let style_a = a.compare_style(request);
        let style_b = b.compare_style(request);

        if let Some(ord) = Self::compare_stage(style_a, style_b) {
            return ord;
        }

        let weight_a = a.compare_weight(request);
        let weight_b = b.compare_weight(request);

        if let Some(ord) = Self::compare_stage(weight_a, weight_b) {
            return ord;
        }

        let stretch_a = a.compare_stretch(request);
        let stretch_b = b.compare_stretch(request);

        if let Some(ord) = Self::compare_stage(stretch_a, stretch_b) {
            return ord;
        }

        Ordering::Equal
    }

    /// Returns the best match in `font_faces`, or None if no strong match found.
    pub(crate) fn match_font_face_rule<'a>(
        font_faces: &'a [FontFaceRule],
        request: &FontRequest,
    ) -> Option<&'a FontFaceRule> {
        font_faces
            .iter()
            .max_by(|a, b| Self::compare_face(*a, *b, request))
    }

    /// Returns the best match in `faces`, filtered for the specified family.
    pub(crate) fn match_system_face<'a>(
        faces: impl Iterator<Item = &'a fontdb::FaceInfo>,
        family: &str,
        request: &FontRequest,
    ) -> Option<fontdb::ID> {
        faces
            .filter(|face| {
                face.families
                    .iter()
                    .any(|(name, _)| name.eq_ignore_ascii_case(family))
            })
            .max_by(|a, b| Self::compare_face(*a, *b, request))
            .map(|face| face.id)
    }
}

// FontCandidate impl for FontFaceRule (CSS face)
impl FontCandidate for FontFaceRule {
    fn compare_style(&self, request: &FontRequest) -> StyleMatch {
        use crate::FontStyle;
        use css::rule::FontStyle as FaceStyle;

        let candidate_style = &self.style;
        match (&request.style, candidate_style) {
            // Normal
            (FontStyle::Normal, FaceStyle::Normal) => StyleMatch {
                category: StyleCategory::Exact,
                distance: 0.0,
            },
            // Italic
            (FontStyle::Italic, FaceStyle::Italic) => StyleMatch {
                category: StyleCategory::Exact,
                distance: 0.0,
            },
            // Oblique
            (FontStyle::Oblique(req_angle), FaceStyle::Oblique(prov_oblique)) => {
                let req = *req_angle;
                match prov_oblique {
                    ObliqueStyle::Single(angle) => {
                        let prov = angle.to_degrees();
                        StyleMatch {
                            category: StyleCategory::Exact,
                            distance: (prov - req).abs(),
                        }
                    }
                    ObliqueStyle::Range(min, max) => {
                        let min = min.to_degrees();
                        let max = max.to_degrees();
                        if req < min {
                            StyleMatch {
                                category: StyleCategory::Exact,
                                distance: min - req,
                            }
                        } else if req > max {
                            StyleMatch {
                                category: StyleCategory::Exact,
                                distance: req - max,
                            }
                        } else {
                            StyleMatch {
                                category: StyleCategory::Exact,
                                distance: 0.0,
                            }
                        }
                    }
                }
            }
            // Italic requested, Oblique provided (any kind)
            (FontStyle::Italic, FaceStyle::Oblique(_)) => StyleMatch {
                category: StyleCategory::ItalicOblique,
                distance: 0.0,
            },
            // Oblique requested, Italic provided
            (FontStyle::Oblique(_), FaceStyle::Italic) => StyleMatch {
                category: StyleCategory::ItalicOblique,
                distance: 0.0,
            },
            // Normal requested, Oblique or Italic provided
            (FontStyle::Normal, FaceStyle::Italic) | (FontStyle::Normal, FaceStyle::Oblique(_)) => {
                StyleMatch {
                    category: StyleCategory::NormalFallback,
                    distance: 0.0,
                }
            }
            _ => StyleMatch {
                category: StyleCategory::NoMatch,
                distance: f32::MAX,
            },
        }
    }

    fn compare_weight(&self, request: &FontRequest) -> WeightMatch {
        use css::rule::FontWeight;
        let req = request.weight.0;
        let candidate_weight = &self.weight;
        let distance = match candidate_weight {
            FontWeight::Single(w) => (req as f32 - *w as f32).abs(),
            FontWeight::Range { min, max } => {
                let req = req as f32;
                let min = *min as f32;
                let max = *max as f32;
                if req < min {
                    min - req
                } else if req > max {
                    req - max
                } else {
                    0.0
                }
            }
        };
        WeightMatch { distance }
    }

    fn compare_stretch(&self, request: &FontRequest) -> StretchMatch {
        use css::rule::FontStretch;
        let req = request.stretch.0;
        let candidate_stretch = &self.stretch;
        // CSS values as percent numbers, where applicable
        let candidate_percent = |s: &FontStretch| match s {
            FontStretch::UltraCondensed => 50.0,
            FontStretch::ExtraCondensed => 62.5,
            FontStretch::Condensed => 75.0,
            FontStretch::SemiCondensed => 87.5,
            FontStretch::Normal => 100.0,
            FontStretch::SemiExpanded => 112.5,
            FontStretch::Expanded => 125.0,
            FontStretch::ExtraExpanded => 150.0,
            FontStretch::UltraExpanded => 200.0,
            _ => unreachable!(),
        };
        let distance = match candidate_stretch {
            s @ FontStretch::UltraCondensed
            | s @ FontStretch::ExtraCondensed
            | s @ FontStretch::Condensed
            | s @ FontStretch::SemiCondensed
            | s @ FontStretch::Normal
            | s @ FontStretch::SemiExpanded
            | s @ FontStretch::Expanded
            | s @ FontStretch::ExtraExpanded
            | s @ FontStretch::UltraExpanded => (candidate_percent(s) - req).abs(),
            FontStretch::Range { min, max } => {
                if req < *min {
                    min - req
                } else if req > *max {
                    req - max
                } else {
                    0.0
                }
            }
        };
        StretchMatch { distance }
    }
}

// FontCandidate impl for fontdb::FaceInfo (System face)
impl FontCandidate for fontdb::FaceInfo {
    fn compare_style(&self, request: &FontRequest) -> StyleMatch {
        use crate::FontStyle;
        use fontdb::Style;
        // fontdb doesn't distinguish between italic and oblique, nor expose angle
        let (category, distance) = match (&request.style, &self.style) {
            (FontStyle::Normal, Style::Normal) => (StyleCategory::Exact, 0.0),
            (FontStyle::Italic, Style::Italic) => (StyleCategory::Exact, 0.0),
            (FontStyle::Oblique(_), Style::Oblique) => {
                // Don't invent an angle, but represent as "unknown" (distance=0.0)
                (StyleCategory::Exact, 0.0)
            }
            // Italic requested, Oblique provided or vice versa
            (FontStyle::Italic, Style::Oblique) | (FontStyle::Oblique(_), Style::Italic) => {
                (StyleCategory::ItalicOblique, 0.0)
            }
            // Normal requested, Italic/Oblique provided
            (FontStyle::Normal, Style::Italic) | (FontStyle::Normal, Style::Oblique) => {
                (StyleCategory::NormalFallback, 0.0)
            }
            _ => (StyleCategory::NoMatch, f32::MAX),
        };
        StyleMatch { category, distance }
    }

    fn compare_weight(&self, request: &FontRequest) -> WeightMatch {
        let req = request.weight.0;
        let candidate = self.weight.0;
        let distance = (req as f32 - candidate as f32).abs();
        WeightMatch { distance }
    }

    fn compare_stretch(&self, request: &FontRequest) -> StretchMatch {
        // .stretch.to_number() yields e.g. 100.0 for normal
        let req = request.stretch.0;
        let candidate = self.stretch.to_number() as f32;
        let distance = (req - candidate).abs();
        StretchMatch { distance }
    }
}
