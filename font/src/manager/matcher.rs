use crate::FontRequest;
use css::{FontFaceRule, ObliqueStyle};
use std::cmp::Ordering;

pub(crate) struct FontMatcher;

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
            ord => ord.reverse(), // Lower enum value is better
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

impl FontMatcher {
    fn compare_stage<T: Ord>(a: T, b: T) -> Option<Ordering> {
        match a.cmp(&b) {
            Ordering::Equal => None,
            ord => Some(ord),
        }
    }
    /// Compares two faces for the given request, returning Ordering as per the CSS font matching algorithm.
    fn compare_face(a: &FontFaceRule, b: &FontFaceRule, request: &FontRequest) -> Ordering {
        let style_a = Self::compare_style(&request.style, &a.style);
        let style_b = Self::compare_style(&request.style, &b.style);

        if let Some(ord) = Self::compare_stage(style_a, style_b) {
            return ord;
        }

        let weight_a = Self::compare_weight(request.weight.0, &a.weight);
        let weight_b = Self::compare_weight(request.weight.0, &b.weight);

        if let Some(ord) = Self::compare_stage(weight_a, weight_b) {
            return ord;
        }

        let stretch_a = Self::compare_stretch(request.stretch.0, &a.stretch);
        let stretch_b = Self::compare_stretch(request.stretch.0, &b.stretch);

        if let Some(ord) = Self::compare_stage(stretch_a, stretch_b) {
            return ord;
        }

        Ordering::Equal
    }

    /// Compares the requested style to a candidate face's style, returning a StyleMatch.
    fn compare_style(requested: &crate::FontStyle, provided: &css::rule::FontStyle) -> StyleMatch {
        use css::rule::FontStyle as FaceStyle;

        let (category, distance) = match (requested, provided) {
            // Exact match: Normal
            (crate::FontStyle::Normal, FaceStyle::Normal) => (StyleCategory::Exact, 0.0),
            // Exact match: Italic
            (crate::FontStyle::Italic, FaceStyle::Italic) => (StyleCategory::Exact, 0.0),
            // Exact match: Oblique
            (crate::FontStyle::Oblique(req_obl), FaceStyle::Oblique(face_obl)) => {
                let d = Self::oblique_distance(req_obl, face_obl);
                (StyleCategory::Exact, d)
            }
            // Italic requested, Oblique provided
            (crate::FontStyle::Italic, FaceStyle::Oblique(_)) => {
                (StyleCategory::ItalicOblique, 0.0)
            }
            // Oblique requested, Italic provided
            (crate::FontStyle::Oblique(_), FaceStyle::Italic) => {
                (StyleCategory::ItalicOblique, 0.0)
            }
            // Normal requested, Oblique or Italic provided
            (crate::FontStyle::Normal, FaceStyle::Oblique(_))
            | (crate::FontStyle::Normal, FaceStyle::Italic) => (StyleCategory::NormalFallback, 0.0),
            // No match
            _ => (StyleCategory::NoMatch, f32::MAX),
        };

        StyleMatch { category, distance }
    }

    /// Compares the requested weight to a candidate face's weight, returning a WeightMatch.
    fn compare_weight(
        requested_weight: u16,
        provided_weight: &css::rule::FontWeight,
    ) -> WeightMatch {
        use css::rule::FontWeight;

        let distance = match provided_weight {
            FontWeight::Single(provided) => (requested_weight as f32 - *provided as f32).abs(),
            FontWeight::Range { min, max } => {
                let requested_f = requested_weight as f32;
                let min_f = *min as f32;
                let max_f = *max as f32;
                if requested_f < min_f {
                    min_f - requested_f
                } else if requested_f > max_f {
                    requested_f - max_f
                } else {
                    0.0
                }
            }
        };

        WeightMatch { distance }
    }

    /// Compares the requested stretch to a candidate face's stretch, returning a StretchMatch.
    fn compare_stretch(
        requested_stretch: f32,
        provided_stretch: &css::rule::FontStretch,
    ) -> StretchMatch {
        use css::rule::FontStretch;

        let distance = match provided_stretch {
            FontStretch::UltraCondensed => (requested_stretch - 50.0).abs(),
            FontStretch::ExtraCondensed => (requested_stretch - 62.5).abs(),
            FontStretch::Condensed => (requested_stretch - 75.0).abs(),
            FontStretch::SemiCondensed => (requested_stretch - 87.5).abs(),
            FontStretch::Normal => (requested_stretch - 100.0).abs(),
            FontStretch::SemiExpanded => (requested_stretch - 112.5).abs(),
            FontStretch::Expanded => (requested_stretch - 125.0).abs(),
            FontStretch::ExtraExpanded => (requested_stretch - 150.0).abs(),
            FontStretch::UltraExpanded => (requested_stretch - 200.0).abs(),
            FontStretch::Range { min, max } => {
                let min_f = *min;
                let max_f = *max;
                if requested_stretch < min_f {
                    min_f - requested_stretch
                } else if requested_stretch > max_f {
                    requested_stretch - max_f
                } else {
                    0.0
                }
            }
        };

        StretchMatch { distance }
    }

    fn oblique_distance(requested: &f32, provided: &ObliqueStyle) -> f32 {
        let req_angle = *requested;
        match provided {
            ObliqueStyle::Single(angle) => {
                let prov_angle = angle.to_degrees();
                (prov_angle - req_angle).abs()
            }
            ObliqueStyle::Range(min, max) => {
                let min_deg = min.to_degrees();
                let max_deg = max.to_degrees();
                if req_angle >= min_deg && req_angle <= max_deg {
                    0.0
                } else if req_angle < min_deg {
                    min_deg - req_angle
                } else {
                    req_angle - max_deg
                }
            }
        }
    }

    /// Returns the best match in `font_faces`, or None if no strong match is found.
    pub(crate) fn match_font_face<'a>(
        font_faces: &'a [FontFaceRule],
        request: &FontRequest,
    ) -> Option<&'a FontFaceRule> {
        font_faces
            .iter()
            .max_by(|a, b| Self::compare_face(a, b, request))
    }
}
