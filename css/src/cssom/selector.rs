/// Operators for attribute selectors, e.g. [attr=value], [attr^="foo"], etc.
#[derive(Debug, Clone, PartialEq)]
pub enum AttrSelectorOperator {
    Equals,         // =
    Includes,       // ~=
    DashMatch,      // |=
    PrefixMatch,    // ^=
    SuffixMatch,    // $=
    SubstringMatch, // *=
}

/// Represents an indivisible simple selector.
#[derive(Debug, Clone, PartialEq)]
pub enum SimpleSelector {
    /// Element/tag selector: div, body, etc.
    Type(String),

    /// Class selector: .my-class
    Class(String),

    /// Id selector: #myid
    Id(String),

    /// Universal selector: *
    Universal,

    /// Attribute selector: [attr=value], [attr]
    Attribute {
        name: String,
        value: Option<String>,
        operator: Option<AttrSelectorOperator>,
    },

    /// Pseudo-class selector (optionally with argument): :hover, :nth-child(2)
    PseudoClass(String, Option<Vec<ComplexSelector>>),

    /// Pseudo-element selector: ::before, ::after
    PseudoElement(String),
}

/// A compound selector: one or more simple selectors without combinators.
/// e.g. `div.foo#main[attr]:hover::before` is a single CompoundSelector.
#[derive(Debug, Clone, PartialEq)]
pub struct CompoundSelector {
    pub simple_selectors: Vec<SimpleSelector>,
}

impl CompoundSelector {
    pub fn new() -> Self {
        Self {
            simple_selectors: Vec::new(),
        }
    }
    pub fn push(&mut self, selectors: SimpleSelector) {
        self.simple_selectors.push(selectors);
    }
}

/// The four combinators supported by CSS.
#[derive(Debug, Clone, PartialEq)]
pub enum Combinator {
    Descendant,      // (whitespace)
    Child,           // '>'
    AdjacentSibling, // '+'
    GeneralSibling,  // '~'
}

/// A complex selector: connects compound selectors using combinators.
/// e.g. `div.foo > p.bar + span:hover`
#[derive(Debug, Clone, PartialEq)]
pub struct ComplexSelector {
    pub first: CompoundSelector,
    pub rest: Vec<(Combinator, CompoundSelector)>,
}
