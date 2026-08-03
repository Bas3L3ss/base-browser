use std::collections::HashMap;

use html::DomElement;

use crate::{
    CssOm, CssRule, Declaration, Origin, Parser, Property, RuleRef, Specificity, Tokenizer,
    cssom::css_value::CssValue, style::computed_style::ComputedStyle,
};

/// Owns just the relevant info rather than borrowing an entire Declaration.
/// This allows merging borrowed (stylesheet) and owned (inline) variants.
#[derive(Debug, Clone)]
struct DeclarationCandidate {
    property: Property,
    value: CssValue,
    origin: Origin,
    specificity: Specificity,
    source_order: u32,
}

impl DeclarationCandidate {
    pub fn new(
        property: Property,
        value: CssValue,
        origin: Origin,
        specificity: Specificity,
        source_order: u32,
    ) -> Self {
        Self {
            property,
            value,
            origin,
            specificity,
            source_order,
        }
    }

    pub fn outranks(&self, other: &Self) -> bool {
        if self.value.is_important != other.value.is_important {
            return self.value.is_important;
        }

        if self.origin != other.origin {
            return self.origin > other.origin;
        }

        if self.specificity != other.specificity {
            return self.specificity > other.specificity;
        }

        self.source_order > other.source_order
    }
}

pub struct CascadeSource<'a> {
    pub origin: Origin,
    pub matches: Option<Vec<RuleRef>>,
    pub cssom: &'a CssOm,
}

pub struct Cascade;

impl Cascade {
    pub(crate) fn compute(element: &DomElement, sources: &[CascadeSource<'_>]) -> ComputedStyle {
        let mut winners: HashMap<Property, DeclarationCandidate> = HashMap::new();

        // Apply each source (user agent, author) in order
        for source in sources {
            if let Some(ref rule_matches) = source.matches {
                Self::apply_matches(&mut winners, rule_matches, source.cssom, source.origin);
            }
        }

        // Inline style (highest specificity, Inline origin, not important)
        if let Some(style) = &element.attributes.style {
            let tokens = Tokenizer::from_string(style);
            let mut parser = Parser::new(&tokens);
            let declarations = parser.parse_declarations();

            Self::apply_inline(&mut winners, &declarations);
        }

        // Produce output ComputedStyle with properties from the best candidates
        let mut properties = HashMap::new();

        for (_, candidate) in winners {
            properties.insert(candidate.property, candidate.value);
        }

        ComputedStyle { properties }
    }

    fn consider_candidate(
        winners: &mut HashMap<Property, DeclarationCandidate>,
        candidate: DeclarationCandidate,
    ) {
        match winners.get(&candidate.property) {
            Some(existing) => {
                if candidate.outranks(existing) {
                    winners.insert(candidate.property.clone(), candidate);
                }
            }
            None => {
                winners.insert(candidate.property.clone(), candidate);
            }
        }
    }

    fn apply_matches(
        winners: &mut HashMap<Property, DeclarationCandidate>,
        matches: &[RuleRef],
        cssom: &CssOm,
        origin: Origin,
    ) {
        for matched in matches {
            if let Some(rule) = cssom.rules.get(matched.rule_index as usize) {
                if let CssRule::Style(style_rule) = rule {
                    if let Some(complex_selector) =
                        style_rule.selectors.get(matched.selector_index as usize)
                    {
                        for declaration in &style_rule.declarations {
                            let candidate = DeclarationCandidate::new(
                                declaration.property.clone(),
                                declaration.value.clone(),
                                origin,
                                complex_selector.specificity,
                                matched.rule_index,
                            );
                            Self::consider_candidate(winners, candidate);
                        }
                    }
                }
            }
        }
    }

    fn apply_inline(
        winners: &mut HashMap<Property, DeclarationCandidate>,
        declarations: &[Declaration],
    ) {
        for declaration in declarations {
            let candidate = DeclarationCandidate::new(
                declaration.property.clone(),
                declaration.value.clone(),
                Origin::Inline,
                Specificity::INLINE,
                0,
            );
            Self::consider_candidate(winners, candidate);
        }
    }
}
