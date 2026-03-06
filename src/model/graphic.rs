//! Graphic unit objects (CT_GraphicUnit subtypes).

use crate::types::StBox;

/// A graphic object within a page layer.
#[derive(Debug)]
pub enum GraphicObject {
    Image(ImageObject),
    // Future: Text(TextObject), Path(PathObject)
}

/// ImageObject — displays a referenced image resource.
///
/// Attributes from CT_GraphicUnit (base):
/// - ID: unique identifier (ST_ID, required)
/// - Boundary: bounding box (ST_Box, required)
///
/// ImageObject-specific:
/// - ResourceID: reference to MultiMedia resource (ST_RefID, required)
#[derive(Debug)]
pub struct ImageObject {
    pub id: u32,
    pub boundary: StBox,
    pub resource_id: u32,
    /// Optional transparency (0-255, default 255 = opaque).
    pub alpha: Option<u8>,
}
