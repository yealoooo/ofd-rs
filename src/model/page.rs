//! Page content structures (Content.xml).

use crate::model::graphic::GraphicObject;

/// A layer within a page content.
#[derive(Debug)]
pub struct Layer {
    pub id: u32,
    /// Layer type: "Body" (default), "Foreground", "Background".
    pub layer_type: Option<String>,
    pub objects: Vec<GraphicObject>,
}

/// Internal page content: one page may have multiple layers.
#[derive(Debug)]
pub(crate) struct PageContent {
    pub layers: Vec<Layer>,
}
