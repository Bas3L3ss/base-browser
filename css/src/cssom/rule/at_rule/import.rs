use crate::{LayerName, MediaQuery};

pub struct Import {
    pub url: Option<String>,
    pub media: Option<MediaQuery>,
    pub layer: Option<LayerName>,
}
