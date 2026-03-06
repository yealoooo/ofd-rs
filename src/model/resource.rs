//! Resource definitions (DocumentRes.xml, PublicRes.xml).

/// Image format for multimedia resources.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    Jpeg,
    Png,
    Bmp,
    Tiff,
}

impl ImageFormat {
    /// File extension (without dot).
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Jpeg => "jpg",
            Self::Png => "png",
            Self::Bmp => "bmp",
            Self::Tiff => "tiff",
        }
    }

    /// MIME type string.
    pub fn mime_type(&self) -> &'static str {
        match self {
            Self::Jpeg => "image/jpeg",
            Self::Png => "image/png",
            Self::Bmp => "image/bmp",
            Self::Tiff => "image/tiff",
        }
    }
}

/// Image data with format and dimensions in mm.
#[derive(Debug, Clone)]
pub struct ImageData {
    pub bytes: Vec<u8>,
    pub format: ImageFormat,
    pub width_mm: f64,
    pub height_mm: f64,
}

impl ImageData {
    /// Create JPEG image data with pixel dimensions and DPI.
    pub fn jpeg(bytes: Vec<u8>, width_px: u32, height_px: u32, dpi: f64) -> Self {
        Self {
            bytes,
            format: ImageFormat::Jpeg,
            width_mm: width_px as f64 * 25.4 / dpi,
            height_mm: height_px as f64 * 25.4 / dpi,
        }
    }

    /// Create image data with pre-calculated mm dimensions.
    pub fn new(bytes: Vec<u8>, format: ImageFormat, width_mm: f64, height_mm: f64) -> Self {
        Self { bytes, format, width_mm, height_mm }
    }
}

/// Internal multimedia resource definition used during assembly.
#[derive(Debug)]
pub(crate) struct MediaDef {
    pub id: u32,
    pub format: ImageFormat,
    /// File name within Res/ directory (e.g. "Image_0.jpg").
    pub file_name: String,
}

/// Internal font definition.
#[derive(Debug)]
pub(crate) struct FontDef {
    pub id: u32,
    pub font_name: String,
    pub family_name: Option<String>,
}
