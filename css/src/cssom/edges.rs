use crate::{ComponentValue, CssValue, Length, Property};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct Edges<T> {
    pub top: T,
    pub right: T,
    pub bottom: T,
    pub left: T,
}

impl<T> Edges<T> {
    pub fn new(top: T, right: T, bottom: T, left: T) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }
}

impl Edges<Length> {
    /// Expands a shorthand array of edge values into explicit top, right, bottom, left.
    ///
    /// Accepts an array of Lengths (1 to 4 values) per CSS shorthand rules.
    /// If None or array is empty, all edges use the provided initial value.
    pub fn expand_from_shorthand(values: Option<&[Length]>, initial: Length) -> Self {
        match values {
            Some([v1]) => Self::new(v1.clone(), v1.clone(), v1.clone(), v1.clone()),
            Some([v1, v2]) => Self::new(v1.clone(), v2.clone(), v1.clone(), v2.clone()),
            Some([v1, v2, v3]) => Self::new(v1.clone(), v2.clone(), v3.clone(), v2.clone()),
            Some([v1, v2, v3, v4]) => Self::new(v1.clone(), v2.clone(), v3.clone(), v4.clone()),
            _ => Self::new(initial.clone(), initial.clone(), initial.clone(), initial),
        }
    }

    /// Combines longhand and shorthand CSS property values into Edges<Length>.
    /// You can use this in ComputedStyle for margin, padding, border, etc.
    pub fn from_properties(
        properties: &HashMap<Property, CssValue>,
        top: Property,
        right: Property,
        bottom: Property,
        left: Property,
        shorthand: Property,
        initial: Length,
    ) -> Self {
        let shorthand_vals: Option<Vec<Length>> = properties.get(&shorthand).map(|v| {
            v.components
                .iter()
                .filter_map(ComponentValue::as_length)
                .collect()
        });

        let expanded = Edges::expand_from_shorthand(shorthand_vals.as_deref(), initial.clone());

        // Helper to get the first Length from a property (if present)
        let get_single_length = |prop: &Property| {
            properties
                .get(prop)
                .and_then(|v| v.components.first())
                .and_then(ComponentValue::as_length)
        };

        Self::new(
            get_single_length(&top).unwrap_or(expanded.top),
            get_single_length(&right).unwrap_or(expanded.right),
            get_single_length(&bottom).unwrap_or(expanded.bottom),
            get_single_length(&left).unwrap_or(expanded.left),
        )
    }
}
