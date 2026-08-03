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
    pub const INLINE: Self = Self {
        ids: 1,
        classes: 0,
        types: 0,
    };

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

#[cfg(test)]
mod tests {
    use std::{assert_eq, vec};

    use super::*;
    use crate::cssom::{
        component_value::ComponentValue,
        selector::{Combinator, ComplexSelector, CompoundSelector, PseudoArg, SimpleSelector},
    };

    // --- Helper functions for constructing selectors ---
    fn compound(ss: Vec<SimpleSelector>) -> CompoundSelector {
        let mut c = CompoundSelector::new();
        for s in ss {
            c.push(s);
        }
        c
    }

    fn complex(
        first: CompoundSelector,
        rest: Vec<(Combinator, CompoundSelector)>,
    ) -> ComplexSelector {
        ComplexSelector::new(first, rest)
    }

    fn pseudo_list(selectors: Vec<ComplexSelector>) -> PseudoArg {
        PseudoArg::SelectorList(selectors)
    }

    fn id_selector() -> SimpleSelector {
        SimpleSelector::Id("foo".into())
    }

    fn class_selector() -> SimpleSelector {
        SimpleSelector::Class("foo".into())
    }

    fn type_selector() -> SimpleSelector {
        SimpleSelector::Type("div".into())
    }

    fn universal_selector() -> SimpleSelector {
        SimpleSelector::Universal
    }

    fn attr_selector() -> SimpleSelector {
        SimpleSelector::Attribute {
            name: "a".to_string(),
            operator: None,
            value: None,
        }
    }

    fn pseudo_element_selector() -> SimpleSelector {
        SimpleSelector::PseudoElement("before".into())
    }

    fn pseudo_class_selector() -> SimpleSelector {
        SimpleSelector::PseudoClass("hover".into(), None)
    }

    fn simple(css: SimpleSelector) -> CompoundSelector {
        compound(vec![css])
    }

    // --- Tests ---

    #[test]
    fn test_simple_selectors_specificity() {
        // Type selector: "div"
        assert_eq!(
            Specificity::compute_compound_selector(&simple(type_selector())),
            Specificity {
                ids: 0,
                classes: 0,
                types: 1
            }
        );

        // Universal selector: "*"
        assert_eq!(
            Specificity::compute_compound_selector(&simple(universal_selector())),
            Specificity {
                ids: 0,
                classes: 0,
                types: 0
            }
        );

        // ID selector: "#foo"
        assert_eq!(
            Specificity::compute_compound_selector(&simple(id_selector())),
            Specificity {
                ids: 1,
                classes: 0,
                types: 0
            }
        );

        // Class selector: ".foo"
        assert_eq!(
            Specificity::compute_compound_selector(&simple(class_selector())),
            Specificity {
                ids: 0,
                classes: 1,
                types: 0
            }
        );

        // Attribute selector: [a]
        assert_eq!(
            Specificity::compute_compound_selector(&simple(attr_selector())),
            Specificity {
                ids: 0,
                classes: 1,
                types: 0
            }
        );

        // Pseudo-element selector: ::before
        assert_eq!(
            Specificity::compute_compound_selector(&simple(pseudo_element_selector())),
            Specificity {
                ids: 0,
                classes: 0,
                types: 1
            }
        );

        // Normal (non-functional) pseudo-class: :hover
        assert_eq!(
            Specificity::compute_compound_selector(&simple(pseudo_class_selector())),
            Specificity {
                ids: 0,
                classes: 1,
                types: 0
            }
        );
    }

    #[test]
    fn test_compound_selectors_specificity() {
        // "div.foo#id[attr]:hover::before" = type (1) + class (1) + id (1) + attr (1) + pseudo-class (1) + pseudo-element (1 type)
        let s = compound(vec![
            type_selector(),           // 1 type
            class_selector(),          // 1 class
            id_selector(),             // 1 id
            attr_selector(),           // 1 class
            pseudo_class_selector(),   // 1 class
            pseudo_element_selector(), // 1 type
        ]);
        assert_eq!(
            Specificity::compute_compound_selector(&s),
            Specificity {
                ids: 1,
                classes: 3,
                types: 2
            }
        );
    }

    #[test]
    fn test_complex_selectors_specificity_combinator_no_effect() {
        // "div > .foo .bar#id" = div (type=1), then .foo (class=1), .bar (class=1), #id (id=1)
        let s = complex(
            simple(type_selector()), // [div]
            vec![
                (Combinator::Child, simple(class_selector())), // > .foo
                (
                    Combinator::Descendant,
                    compound(vec![
                        class_selector(),
                        id_selector(), // .bar#id
                    ]),
                ),
            ],
        );
        assert_eq!(
            s.specificity,
            Specificity {
                ids: 1,
                classes: 2,
                types: 1
            }
        );

        // Check all combinators: they should NOT change specificity
        let s2 = complex(
            simple(type_selector()), // div
            vec![
                (Combinator::AdjacentSibling, simple(attr_selector())), // + [a]
                (Combinator::GeneralSibling, simple(pseudo_class_selector())), // ~ :hover
            ],
        );
        assert_eq!(
            s2.specificity,
            Specificity {
                ids: 0,
                classes: 2,
                types: 1
            }
        );
    }

    #[test]
    fn test_functional_pseudo_classes_is_not_has() {
        // :is(.foo, #id, div[attr])
        let pseudo = SimpleSelector::PseudoClass(
            "is".into(),
            Some(pseudo_list(vec![
                complex(simple(class_selector()), vec![]), // .foo: class (0,1,0)
                complex(simple(id_selector()), vec![]),    // #id:   id (1,0,0)
                complex(compound(vec![type_selector(), attr_selector()]), vec![]), // div[attr]: type+class (0,1,1)
            ])),
        );
        // For :is(), specificity is max of arg selectors. Here: #id is (1,0,0)
        assert_eq!(
            Specificity::compute_simple_selector(&pseudo),
            Specificity {
                ids: 1,
                classes: 0,
                types: 0
            }
        );

        // :not(div, .foo[attr])
        let pseudo = SimpleSelector::PseudoClass(
            "not".into(),
            Some(pseudo_list(vec![
                complex(simple(type_selector()), vec![]), // div: (0,0,1)
                complex(compound(vec![class_selector(), attr_selector()]), vec![]), // .foo[attr]: (0,2,0)
            ])),
        );
        // max is (0,2,1)
        assert_eq!(
            Specificity::compute_simple_selector(&pseudo),
            Specificity {
                ids: 0,
                classes: 2,
                types: 0
            }
        );

        // :has(#a.b)
        let pseudo = SimpleSelector::PseudoClass(
            "has".into(),
            Some(pseudo_list(vec![complex(
                compound(vec![id_selector(), class_selector()]),
                vec![],
            )])),
        );
        assert_eq!(
            Specificity::compute_simple_selector(&pseudo),
            Specificity {
                ids: 1,
                classes: 1,
                types: 0
            }
        );

        // :where(#id, .foo, div[attr]) -- always zero
        let pseudo = SimpleSelector::PseudoClass(
            "where".into(),
            Some(pseudo_list(vec![
                complex(simple(class_selector()), vec![]),
                complex(simple(id_selector()), vec![]),
                complex(compound(vec![type_selector(), attr_selector()]), vec![]),
            ])),
        );
        assert_eq!(
            Specificity::compute_simple_selector(&pseudo),
            Specificity::ZERO
        )
    }

    #[test]
    fn test_nested_functional_pseudo_classes() {
        // :is(:not(#a, .foo), .bar)
        let nested = SimpleSelector::PseudoClass(
            "is".into(),
            Some(pseudo_list(vec![
                complex(
                    compound(vec![SimpleSelector::PseudoClass(
                        "not".into(),
                        Some(pseudo_list(vec![
                            complex(simple(id_selector()), vec![]),    // #a: (1,0,0)
                            complex(simple(class_selector()), vec![]), // .foo: (0,1,0)
                        ])),
                    )]),
                    vec![],
                ),
                complex(simple(class_selector()), vec![]), // .bar
            ])),
        );
        // :not() max is (1,0,0), .bar is (0,1,0) - so :is() returns (1,0,0)
        assert_eq!(
            Specificity::compute_simple_selector(&nested),
            Specificity {
                ids: 1,
                classes: 0,
                types: 0
            }
        );

        // Deeply nested: :is(:not(:has(div)), :where(#id))
        let deep = SimpleSelector::PseudoClass(
            "is".into(),
            Some(pseudo_list(vec![
                complex(
                    compound(vec![SimpleSelector::PseudoClass(
                        "not".into(),
                        Some(pseudo_list(vec![complex(
                            compound(vec![SimpleSelector::PseudoClass(
                                "has".into(),
                                Some(pseudo_list(vec![
                                    complex(simple(type_selector()), vec![]), // div: (0,0,1)
                                ])),
                            )]),
                            vec![],
                        )])),
                    )]),
                    vec![],
                ),
                complex(
                    compound(vec![SimpleSelector::PseudoClass(
                        "where".into(),
                        Some(pseudo_list(vec![complex(simple(id_selector()), vec![])])),
                    )]),
                    vec![],
                ),
            ])),
        );
        // :has(div) = (0,0,1), :not(:has(...)) = (0,0,1), :where(#id)=0, so max is (0,0,1)
        assert_eq!(
            Specificity::compute_simple_selector(&deep),
            Specificity {
                ids: 0,
                classes: 0,
                types: 1
            }
        );
    }

    #[test]
    fn test_unknown_pseudo_classes() {
        // Non-functional unknown pseudo-class: :foobar
        let sel = SimpleSelector::PseudoClass("foobar".into(), None);
        assert_eq!(
            Specificity::compute_simple_selector(&sel),
            Specificity {
                ids: 0,
                classes: 1,
                types: 0
            }
        );

        // Functional but unknown pseudo-class: :foobar(...)
        let sel = SimpleSelector::PseudoClass(
            "foobar".into(),
            Some(PseudoArg::Values(vec![ComponentValue::Ident(
                "somearg".to_string(),
            )])),
        );
        // Should count as a class selector (0,1,0)
        assert_eq!(
            Specificity::compute_simple_selector(&sel),
            Specificity {
                ids: 0,
                classes: 1,
                types: 0
            }
        );
    }

    #[test]
    fn test_selector_list_edge_cases() {
        // Empty selector list: max should be zero
        assert_eq!(Specificity::compute_selector_list(&[]), Specificity::ZERO);

        // Single selector
        let one = vec![complex(simple(type_selector()), vec![])]; // div: (0,0,1)
        assert_eq!(
            Specificity::compute_selector_list(&one),
            Specificity {
                ids: 0,
                classes: 0,
                types: 1
            }
        );

        // Multiple, use max
        let many = vec![
            complex(simple(type_selector()), vec![]),  // (0,0,1)
            complex(simple(id_selector()), vec![]),    // (1,0,0)
            complex(simple(class_selector()), vec![]), // (0,1,0)
        ];
        assert_eq!(
            Specificity::compute_selector_list(&many),
            Specificity {
                ids: 1,
                classes: 0,
                types: 0
            }
        );
    }

    #[test]
    fn test_ordering_and_equality() {
        // ID > Class > Type
        let id = Specificity {
            ids: 1,
            classes: 0,
            types: 0,
        };
        let class = Specificity {
            ids: 0,
            classes: 1,
            types: 0,
        };
        let typ = Specificity {
            ids: 0,
            classes: 0,
            types: 1,
        };
        let id2 = Specificity {
            ids: 2,
            classes: 0,
            types: 0,
        };
        let c2t1 = Specificity {
            ids: 0,
            classes: 2,
            types: 1,
        };
        // lexicographic
        assert!(id > class);
        assert!(class > typ);
        assert!(id2 > id);
        assert!(c2t1 > class);
        assert!(c2t1 > typ);
        assert_eq!(
            id,
            Specificity {
                ids: 1,
                classes: 0,
                types: 0
            }
        );
        assert_eq!(typ, typ);
    }

    #[test]
    fn test_add_assign_specificity() {
        let mut s = Specificity::ZERO;
        s += Specificity {
            ids: 1,
            classes: 0,
            types: 1,
        };
        s += Specificity {
            ids: 0,
            classes: 2,
            types: 0,
        };
        assert_eq!(
            s,
            Specificity {
                ids: 1,
                classes: 2,
                types: 1
            }
        );
    }

    #[test]
    fn test_regressions_realistic_selectors() {
        // "#foo.bar[attr]:hover > div::before:is(.a, #b) .c[attr2='v']:active"
        // Specificity breakdown:
        // #foo        (1,0,0)
        // .bar        (0,1,0)
        // [attr]      (0,1,0)
        // :hover      (0,1,0)
        // div         (0,0,1)
        // ::before    (0,0,1)
        // :is(.a, #b) max(.a:(0,1,0), #b:(1,0,0)) = (1,0,0)
        // .c          (0,1,0)
        // [attr2='v'] (0,1,0)
        // :active     (0,1,0)
        let pseudo_is = SimpleSelector::PseudoClass(
            "is".into(),
            Some(pseudo_list(vec![
                complex(simple(class_selector()), vec![]), // .a
                complex(simple(id_selector()), vec![]),    // #b
            ])),
        );
        let part1 = compound(vec![
            id_selector(),
            class_selector(),
            attr_selector(),
            pseudo_class_selector(),
        ]);
        let part2 = compound(vec![type_selector(), pseudo_element_selector(), pseudo_is]);
        let part3 = compound(vec![
            class_selector(),
            attr_selector(),
            SimpleSelector::PseudoClass("active".into(), None),
        ]);
        // Main complex chain: part1 > part2 . part3
        let cx = complex(
            part1,
            vec![(Combinator::Child, part2), (Combinator::Descendant, part3)],
        );
        // Calc: ids=1 (from #foo) + 1 (from :is) = 2
        // classes=1(.bar)+1([attr])+1(:hover)+1(.c)+1([attr2])+1(:active)=6
        // types=1(div)+1(::before)=2
        assert_eq!(
            cx.specificity,
            Specificity {
                ids: 2,
                classes: 6,
                types: 2
            }
        );
    }

    #[test]
    fn test_regression_deep_nested_functional() {
        // "div:is(.a:not(#b:has(.c)))"
        // div: type=1
        // .a:not(#b:has(.c))
        //    .a: (0,1,0)
        //    :not(...): max of #b:has(.c):
        //       #b: (1,0,0), :has(.c): .c: (0,1,0) => max(1,0,0), (0,1,0) => (1,0,0)
        //   So :not() = (1,0,0). .a and :not() are sibling simple selectors, so type=0, class=1, id=1.

        // :has(.c) as a selector list pseudo argument
        let has_selector = SimpleSelector::PseudoClass(
            "has".into(),
            Some(pseudo_list(vec![
                complex(simple(class_selector()), vec![]), // .c
            ])),
        );
        // #b:has(.c) as a ComplexSelector
        let not_complex = complex(
            compound(vec![
                SimpleSelector::Id("b".into()), // #b
                has_selector,                   // :has(.c)
            ]),
            vec![],
        );
        // :not(#b:has(.c)) with correct pseudo arg (SelectorList)
        let not_selector =
            SimpleSelector::PseudoClass("not".into(), Some(pseudo_list(vec![not_complex])));

        // .a:not(#b:has(.c)) as a CompoundSelector
        let is_compound = compound(vec![
            class_selector(), // .a
            not_selector,     // :not(...)
        ]);

        // :is(.a:not(#b:has(.c))) with correct pseudo arg (SelectorList)
        let is_selector = SimpleSelector::PseudoClass(
            "is".into(),
            Some(pseudo_list(vec![complex(is_compound.clone(), vec![])])),
        );

        // div:is(.a:not(#b:has(.c)))
        let outer = compound(vec![type_selector(), is_selector]);

        assert_eq!(
            Specificity::compute_compound_selector(&outer),
            Specificity {
                ids: 1,
                classes: 2,
                types: 1
            }
        );
    }
}
