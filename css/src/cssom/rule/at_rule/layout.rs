pub struct LayerNode {
    /// Name of this layer
    pub name: Option<String>,

    /// Child layers
    pub children: Vec<LayerNode>,

    /// Cascade order index
    pub order: usize,
}
pub struct LayerName {
    pub path: Vec<String>,
}

pub struct LayerTree {
    pub root: LayerNode,
}
