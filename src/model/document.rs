//! Document root structure (Document.xml).

use crate::types::StBox;

/// Default pixels-per-millimeter ratio, matching ofdrw convention.
/// ofdrw uses `pixels / 5` to convert pixel dimensions to mm.
/// 5 px/mm ≈ 127 DPI.
pub const PPM_DEFAULT: f64 = 5.0;

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

    /// Calculate page size from pixel dimensions using pixels-per-mm ratio.
    /// This matches ofdrw's conversion: `mm = pixels / ppm`.
    /// Use [`PPM_DEFAULT`] (5.0) for ofdrw-compatible behavior.
    pub fn from_pixels_ppm(width_px: u32, height_px: u32, ppm: f64) -> Self {
        Self {
            width_mm: width_px as f64 / ppm,
            height_mm: height_px as f64 / ppm,
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
