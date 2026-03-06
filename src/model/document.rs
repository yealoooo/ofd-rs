//! Document root structure (Document.xml).

use crate::types::StBox;

/// Page size in millimeters.
#[derive(Debug, Clone, Copy)]
pub struct PageSize {
    pub width_mm: f64,
    pub height_mm: f64,
}

impl PageSize {
    pub fn new(width_mm: f64, height_mm: f64) -> Self {
        Self { width_mm, height_mm }
    }

    /// Calculate page size from pixel dimensions and DPI.
    pub fn from_pixels(width_px: u32, height_px: u32, dpi: f64) -> Self {
        Self {
            width_mm: width_px as f64 * 25.4 / dpi,
            height_mm: height_px as f64 * 25.4 / dpi,
        }
    }

    /// Convert to StBox (origin at 0,0).
    pub fn to_box(&self) -> StBox {
        StBox::page(self.width_mm, self.height_mm)
    }
}

/// Internal page definition used during document assembly.
#[derive(Debug)]
pub(crate) struct PageDef {
    pub page_id: u32,
    pub base_loc: String,
}
