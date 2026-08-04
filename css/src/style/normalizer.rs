use std::{collections::HashMap, unreachable, vec};

use crate::{ComponentValue, CssValue, DeclarationCandidate, Property};

pub struct Normalizer;

impl Normalizer {
    pub(crate) fn insert(
        properties: &mut HashMap<Property, CssValue>,
        candidate: DeclarationCandidate,
    ) {
        match candidate.property {
            // Box Model Shorthands
            Property::Margin => {
                Self::expand_and_insert_box_edges(
                    properties,
                    &candidate,
                    Property::MarginTop,
                    Property::MarginRight,
                    Property::MarginBottom,
                    Property::MarginLeft,
                    ComponentValue::Ident(String::from("auto")),
                );
            }
            Property::Padding => {
                Self::expand_and_insert_box_edges(
                    properties,
                    &candidate,
                    Property::PaddingTop,
                    Property::PaddingRight,
                    Property::PaddingBottom,
                    Property::PaddingLeft,
                    ComponentValue::Ident(String::from("0")),
                );
            }
            Property::BorderWidth => {
                // Border width default value is 'medium' per CSS spec.
                Self::expand_and_insert_box_edges(
                    properties,
                    &candidate,
                    Property::BorderTopWidth,
                    Property::BorderRightWidth,
                    Property::BorderBottomWidth,
                    Property::BorderLeftWidth,
                    ComponentValue::Ident(String::from("medium")),
                );
            }
            Property::Border => {
                // Expand border shorthand to Border{Top,Right,Bottom,Left}{Width,Style,Color}
                let border_defaults = (
                    ComponentValue::Ident(String::from("medium")),
                    ComponentValue::Ident(String::from("none")),
                    ComponentValue::Ident(String::from("currentcolor")),
                );

                let (width, style, color) = Self::parse_border_shorthand_components(
                    &candidate.value.components,
                    border_defaults.0.clone(),
                    border_defaults.1.clone(),
                    border_defaults.2.clone(),
                );

                let important = candidate.value.is_important;

                // Helper mapping for sides and their border sub-properties
                let sides = [
                    (
                        Property::BorderTopWidth,
                        Property::BorderTopStyle,
                        Property::BorderTopColor,
                    ),
                    (
                        Property::BorderRightWidth,
                        Property::BorderRightStyle,
                        Property::BorderRightColor,
                    ),
                    (
                        Property::BorderBottomWidth,
                        Property::BorderBottomStyle,
                        Property::BorderBottomColor,
                    ),
                    (
                        Property::BorderLeftWidth,
                        Property::BorderLeftStyle,
                        Property::BorderLeftColor,
                    ),
                ];

                for (width_prop, style_prop, color_prop) in sides {
                    properties.entry(width_prop).or_insert(CssValue {
                        components: vec![width.clone()],
                        is_important: important,
                    });
                    properties.entry(style_prop).or_insert(CssValue {
                        components: vec![style.clone()],
                        is_important: important,
                    });
                    properties.entry(color_prop).or_insert(CssValue {
                        components: vec![color.clone()],
                        is_important: important,
                    });
                }
            }
            Property::BorderTop
            | Property::BorderRight
            | Property::BorderBottom
            | Property::BorderLeft => {
                let border_defaults = (
                    ComponentValue::Ident(String::from("medium")),
                    ComponentValue::Ident(String::from("none")),
                    ComponentValue::Ident(String::from("currentcolor")),
                );
                let border_components = Self::parse_border_logical_components(
                    &candidate.value.components,
                    border_defaults.0.clone(),
                    border_defaults.1.clone(),
                    border_defaults.2.clone(),
                );
                let important = candidate.value.is_important;

                let (width_prop, style_prop, color_prop) = match candidate.property {
                    Property::BorderTop => (
                        Property::BorderTopWidth,
                        Property::BorderTopStyle,
                        Property::BorderTopColor,
                    ),
                    Property::BorderRight => (
                        Property::BorderRightWidth,
                        Property::BorderRightStyle,
                        Property::BorderRightColor,
                    ),
                    Property::BorderBottom => (
                        Property::BorderBottomWidth,
                        Property::BorderBottomStyle,
                        Property::BorderBottomColor,
                    ),
                    Property::BorderLeft => (
                        Property::BorderLeftWidth,
                        Property::BorderLeftStyle,
                        Property::BorderLeftColor,
                    ),
                    _ => unreachable!(),
                };

                for (prop, value) in [
                    (width_prop, &border_components[0]),
                    (style_prop, &border_components[1]),
                    (color_prop, &border_components[2]),
                ] {
                    properties.entry(prop).or_insert(CssValue {
                        components: vec![value.clone()],
                        is_important: important,
                    });
                }
            }
            _ => {
                properties.insert(candidate.property, candidate.value);
            }
        }
    }

    fn parse_border_shorthand_components(
        components: &[ComponentValue],
        default_width: ComponentValue,
        default_style: ComponentValue,
        default_color: ComponentValue,
    ) -> (ComponentValue, ComponentValue, ComponentValue) {
        use crate::cssom::unit::Unit;

        // Border styles per CSS spec
        const BORDER_STYLES: &[&str] = &[
            "none", "hidden", "dotted", "dashed", "solid", "double", "groove", "ridge", "inset",
            "outset",
        ];

        let mut width: Option<ComponentValue> = None;
        let mut style: Option<ComponentValue> = None;
        let mut color: Option<ComponentValue> = None;

        for component in components {
            match component {
                ComponentValue::Ident(s) if BORDER_STYLES.contains(&s.as_str()) => {
                    if style.is_none() {
                        style = Some(component.clone());
                    }
                }
                ComponentValue::Ident(_)
                | ComponentValue::Hash(_)
                | ComponentValue::Function(_)
                    if color.is_none() =>
                {
                    // Accept color as ident, hash, or function (e.g. 'red', '#f00', 'rgb(...)')
                    color = Some(component.clone());
                }
                ComponentValue::Dimension { .. } if width.is_none() => {
                    width = Some(component.clone());
                }
                ComponentValue::Number(n) if width.is_none() => {
                    // Accept bare numbers as 'px' by default (could be refined for zero)
                    width = Some(ComponentValue::Dimension {
                        value: *n,
                        unit: Unit::Px,
                    });
                }
                _ => {}
            }
        }

        (
            width.unwrap_or(default_width),
            style.unwrap_or(default_style),
            color.unwrap_or(default_color),
        )
    }

    fn parse_border_logical_components(
        components: &[ComponentValue],
        default_width: ComponentValue,
        default_style: ComponentValue,
        default_color: ComponentValue,
    ) -> Vec<ComponentValue> {
        let (width, style, color) = Self::parse_border_shorthand_components(
            components,
            default_width,
            default_style,
            default_color,
        );
        vec![width, style, color]
    }

    fn expand_and_insert_box_edges(
        properties: &mut HashMap<Property, CssValue>,
        candidate: &DeclarationCandidate,
        top_prop: Property,
        right_prop: Property,
        bottom_prop: Property,
        left_prop: Property,
        initial: ComponentValue,
    ) {
        let (top, right, bottom, left) =
            Self::expand_shorthand_four_values(&candidate.value.components, initial.clone());

        let insert_edge =
            |properties: &mut HashMap<_, _>, edge_prop: Property, value: ComponentValue| {
                properties.entry(edge_prop).or_insert(CssValue {
                    components: vec![value],
                    is_important: candidate.value.is_important,
                });
            };

        insert_edge(properties, top_prop, top);
        insert_edge(properties, right_prop, right);
        insert_edge(properties, bottom_prop, bottom);
        insert_edge(properties, left_prop, left);
    }

    pub fn expand_shorthand_four_values(
        values: &[ComponentValue],
        initial: ComponentValue,
    ) -> (
        ComponentValue,
        ComponentValue,
        ComponentValue,
        ComponentValue,
    ) {
        match values {
            [v1] => (v1.clone(), v1.clone(), v1.clone(), v1.clone()),
            [v1, v2] => (v1.clone(), v2.clone(), v1.clone(), v2.clone()),
            [v1, v2, v3] => (v1.clone(), v2.clone(), v3.clone(), v2.clone()),
            [v1, v2, v3, v4] => (v1.clone(), v2.clone(), v3.clone(), v4.clone()),

            _ => (initial.clone(), initial.clone(), initial.clone(), initial),
        }
    }
}
