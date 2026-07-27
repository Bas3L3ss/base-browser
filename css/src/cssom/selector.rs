#[derive(Debug, Clone, PartialEq)]
pub enum Selector {
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

    /// Negation selector: :not(selector)
    Not(Box<Selector>),

    /// "Has" pseudo-class (matches if element contains matching descendants): :has(selector)
    Has(Box<Selector>),

    /// "Is" pseudo-class (matches if element matches any of list): :is(selector)
    Is(Vec<Selector>),

    /// "Where" pseudo-class (like :is but no specificity): :where(selector)
    Where(Vec<Selector>),

    /// "Nth-child" pseudo-class: :nth-child(an+b)
    NthChild {
        a: i32,
        b: i32,
        of: Option<Box<Selector>>,
    },

    /// "Nth-of-type" pseudo-class: :nth-of-type(an+b)
    NthOfType {
        a: i32,
        b: i32,
        of: Option<Box<Selector>>,
    },

    /// "Only-child" pseudo-class: :only-child
    OnlyChild,

    /// "Only-of-type" pseudo-class: :only-of-type
    OnlyOfType,

    /// Descendant combinator: E F
    Descendant(Box<Selector>, Box<Selector>),

    /// Child combinator: E > F
    Child(Box<Selector>, Box<Selector>),

    /// Adjacent sibling combinator: E + F
    AdjacentSibling(Box<Selector>, Box<Selector>),

    /// General sibling combinator: E ~ F
    GeneralSibling(Box<Selector>, Box<Selector>),

    /// Pseudo-class selector, e.g. ":hover", ":nth-child(2n+1)"
    /// Example:
    ///   .button:hover { background: blue; }
    ///   li:nth-child(odd) { color: red; }
    PseudoClass(String, Option<String>),

    /// Pseudo-element selector, e.g. "::before", "::after"
    /// Example:
    ///   .item::before { content: "• "; }
    ///   p::after { content: "!"; }
    PseudoElement(String),

    /// Compound selector (multiple without combinators, e.g. div.foo#bar[attr])
    Compound(Vec<Selector>),
}

// Optional: helper enum for attribute selector operator
#[derive(Debug, Clone, PartialEq)]
pub enum AttrSelectorOperator {
    Equals,         // =
    Includes,       // ~=
    DashMatch,      // |=
    PrefixMatch,    // ^=
    SuffixMatch,    // $=
    SubstringMatch, // *=
}
