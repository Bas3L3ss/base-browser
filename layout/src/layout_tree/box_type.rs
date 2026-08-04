#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BoxType {
    Block,
    Inline,
    InlineBlock,
    Text,
    Unknown,
    Anonymous,
    // TODO: Add `Flex`
    // TODO: Add `Grid`
    // TODO: Add `Table`
}

impl BoxType {
    pub fn is_block_level(&self) -> bool {
        match self {
            BoxType::Block => true,
            BoxType::Inline => false,
            BoxType::InlineBlock => false,
            BoxType::Text => false,
            BoxType::Unknown => false,
            BoxType::Anonymous => false,
            // extend here with more cases as needed
        }
    }

    pub fn is_inline_level(&self) -> bool {
        match self {
            BoxType::Inline => true,
            BoxType::InlineBlock => true,
            _ => false,
        }
    }
}
