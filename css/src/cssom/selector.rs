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

    /// Descendant combinator: E F
    Descendant(Box<Selector>, Box<Selector>),

    /// Child combinator: E > F
    Child(Box<Selector>, Box<Selector>),

    /// Adjacent sibling combinator: E + F
    AdjacentSibling(Box<Selector>, Box<Selector>),

    /// General sibling combinator: E ~ F
    GeneralSibling(Box<Selector>, Box<Selector>),

    /// Pseudo-class selector: :hover, :nth-child(), etc.
    PseudoClass(String, Option<String>),

    /// Pseudo-element selector: ::before, ::after, etc.
    PseudoElement(String),

    /// Grouping (comma): E, F, ...
    Group(Vec<Selector>),
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
