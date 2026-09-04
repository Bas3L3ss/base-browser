use crate::style::computed_style::ComputedStyle;
use html::dom::node::Node;

// Optimization / Refactor TODO:
//
// - Use HashMap::entry() API when inserting inherited properties
//   to avoid double lookup with contains_key() + insert().
//
// - Avoid unnecessary clones where ownership can be transferred:
//   - Move CssValue when possible.
//   - Clone only when sharing values between parent and child styles.
//
// - Consider storing computed styles in Vec<Option<ComputedStyle>>
//   indexed by NodeId for O(1) lookup.
//
// - Replace dynamic inherited-property checks with a static table:
//   - Add Property::is_inherited() lookup table.
//   - Avoid match expansion as property count grows.
//
// - Add a static list of inherited properties:
//   - Property::INHERITED_PROPERTIES
//   - Use it when applying inheritance.
//
// - Consider storing initial values in a static property table:
//   - Property -> initial CssValue
//   - Used for initial/unset resolution.
//
// - Consider interning frequently used CSS values:
//   - Colors
//   - Display values
//   - Font values
//   - Keywords.
//
// - Avoid reparsing inline styles repeatedly:
//   - Parse style="" attributes during DOM construction or store parsed declarations.
//
// - Consider using a specialized storage structure instead of HashMap
//   for ComputedStyle if property count becomes fixed:
//   - Enum-indexed array
//   - Bitset + value storage.
//
// - Profile inheritance cost after layout implementation exists before optimizing.

// TODO:
// - Support CSS-wide keywords:
//   - inherit: replace current value with parent's computed value
//   - initial: replace current value with property's initial value
//   - unset: inherit if property is inheritable, otherwise use initial value
//   - revert: restore value from lower cascade origin
//   - revert-layer: restore value from previous cascade layer
//
// - Replace parent.properties iteration with iterating over all CSS properties.
//   This ensures properties like `color: inherit` work even when the parent
//   does not explicitly contain the property.
//
// - Add a complete list/table of inheritable properties.
//
// - Handle inherited custom properties (`--variables`):
//   - Custom properties inherit by default.
//   - Resolve var() during computed value stage.
//
// - Separate specified value and computed value stages:
//   - Cascade should produce specified values.
//   - Inheritance should resolve CSS-wide keywords.
//   - ComputedStyle should contain resolved computed values.
//
// - Handle initial values for properties not specified anywhere.
//
// - Handle property dependencies during computation:
//   - em/rem units depend on font-size.
//   - line-height depends on font metrics.
//   - percentages may depend on parent properties.
//
// - Support property-specific computed value resolution.
//
// - Add support for `all` property:
//   - all: inherit
//   - all: initial
//   - all: unset
//
// - Handle replaced elements and special inheritance cases.
//
// - Add tests:
//   - Basic inherited property.
//   - Non-inherited property.
//   - Explicit `inherit` keyword.
//   - `initial` keyword.
//   - `unset` keyword.
//   - Nested inheritance chain.
//   - Missing parent property.
//   - Custom property inheritance.

/// Handles inheriting CSS properties from parent to child.
pub struct Inheritance;

impl Inheritance {
    pub(crate) fn apply(
        node: &Node,
        style: &mut ComputedStyle,
        styled_nodes: &[Option<ComputedStyle>],
    ) {
        let Some(parent_id) = node.parent else {
            return;
        };

        let Some(parent_style) = styled_nodes[parent_id].as_ref() else {
            return;
        };

        for (property, parent_value) in &parent_style.properties {
            if !property.is_inherited() {
                continue;
            }

            match style.properties.get(property) {
                // Child doesn't specify the property → inherit it.
                None => {
                    style
                        .properties
                        .insert(property.clone(), parent_value.clone());
                }

                // Child explicitly requests inheritance.
                Some(child_value)
                    if child_value.components.len() == 1
                        && child_value.components[0].is_keyword("inherit") =>
                {
                    style
                        .properties
                        .insert(property.clone(), parent_value.clone());
                }

                // Child has its own value.
                _ => {}
            }
        }
    }
}
