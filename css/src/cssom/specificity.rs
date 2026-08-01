use std::ops::AddAssign;

use crate::cssom::selector::{
    Combinator, ComplexSelector, CompoundSelector, PseudoArg, SimpleSelector,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Specificity {
    pub ids: u16,
    pub classes: u16,
    pub types: u16,
}

impl Specificity {
    pub const ZERO: Self = Self {
        ids: 0,
        classes: 0,
        types: 0,
    };

    pub const ID: Self = Self {
        ids: 1,
        classes: 0,
        types: 0,
    };

    pub const CLASS: Self = Self {
        ids: 0,
        classes: 1,
        types: 0,
    };

    pub const TYPE: Self = Self {
        ids: 0,
        classes: 0,
        types: 1,
    };
}

impl AddAssign for Specificity {
    fn add_assign(&mut self, rhs: Self) {
        self.ids += rhs.ids;
        self.classes += rhs.classes;
        self.types += rhs.types;
    }
}

impl Specificity {
    /// Entry point: compute specificity for a ComplexSelector.
    pub fn compute_complex_selector(
        first: &CompoundSelector,
        rest: &[(Combinator, CompoundSelector)],
    ) -> Self {
        let mut spec = Self::compute_compound_selector(first);

        for (_, compound) in rest {
            spec += Self::compute_compound_selector(compound);
        }
        spec
    }

    /// Compute specificity for a CompoundSelector (sum for each SimpleSelector).
    pub fn compute_compound_selector(compound: &CompoundSelector) -> Self {
        let mut spec = Self::default();
        for simple in &compound.simple_selectors {
            spec += Self::compute_simple_selector(simple);
        }
        spec
    }

    /// Compute specificity for a single SimpleSelector.
    pub fn compute_simple_selector(simple: &SimpleSelector) -> Self {
        match simple {
            SimpleSelector::Type(_) => Self::TYPE,
            SimpleSelector::Universal => Self::ZERO,
            SimpleSelector::Id(_) => Self::ID,
            SimpleSelector::Class(_) => Self::CLASS,
            SimpleSelector::Attribute { .. } => Self::CLASS,
            SimpleSelector::PseudoElement(_) => Self::TYPE,
            SimpleSelector::PseudoClass(name, arg) => {
                Self::compute_pseudo_class(name, arg.as_ref())
            }
        }
    }

    /// Handle pseudo-class specificity, including functional forms.
    fn compute_pseudo_class(name: &str, arg: Option<&PseudoArg>) -> Self {
        match (name, arg) {
            // :where() specificity is always zero, regardless of children
            ("where", Some(PseudoArg::SelectorList(_))) => Self::ZERO,
            // :is/:not/:has: max specificity of their selector list arg
            ("is", Some(PseudoArg::SelectorList(list)))
            | ("not", Some(PseudoArg::SelectorList(list)))
            | ("has", Some(PseudoArg::SelectorList(list))) => Self::compute_selector_list(list),
            // Non-functional (no arg), or functional but not in list above: (0,1,0)
            _ => Self::CLASS,
        }
    }

    /// Compute max specificity of a selector list (slice of ComplexSelector).
    pub fn compute_selector_list(list: &[ComplexSelector]) -> Self {
        let mut max = Self::default();
        for selector in list {
            max = max.max(selector.specificity);
        }
        max
    }
}
