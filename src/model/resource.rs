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

    /// Format name for OFD XML (uppercase, matches ofdrw convention).
    pub fn ofd_format(&self) -> &'static str {
        match self {
            Self::Jpeg => "JPEG",
            Self::Png => "PNG",
            Self::Bmp => "BMP",
            Self::Tiff => "TIFF",
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

    /// Detect image format from file header bytes (magic number).
    pub fn detect(data: &[u8]) -> Option<Self> {
        if data.len() < 4 {
            return None;
        }
        if data[0] == 0xFF && data[1] == 0xD8 {
            Some(Self::Jpeg)
        } else if data[0..4] == [0x89, 0x50, 0x4E, 0x47] {
            Some(Self::Png)
        } else if data[0] == 0x42 && data[1] == 0x4D {
            Some(Self::Bmp)
        } else if (data[0] == 0x49 && data[1] == 0x49) || (data[0] == 0x4D && data[1] == 0x4D) {
            Some(Self::Tiff)
        } else {
            None
        }
    }

    /// Detect format from file extension.
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "jpg" | "jpeg" => Some(Self::Jpeg),
            "png" => Some(Self::Png),
            "bmp" => Some(Self::Bmp),
            "tif" | "tiff" => Some(Self::Tiff),
            _ => None,
        }
    }
}

/// Read pixel dimensions from image file header.
/// Returns (width, height) in pixels, or `None` if parsing fails.
pub fn detect_image_dimensions(data: &[u8]) -> Option<(u32, u32)> {
    let format = ImageFormat::detect(data)?;
    match format {
        ImageFormat::Png => detect_png_dimensions(data),
        ImageFormat::Jpeg => detect_jpeg_dimensions(data),
        ImageFormat::Bmp => detect_bmp_dimensions(data),
        ImageFormat::Tiff => None, // TIFF parsing is complex; skip for now
    }
}

fn detect_png_dimensions(data: &[u8]) -> Option<(u32, u32)> {
    // PNG IHDR chunk: signature(8) + length(4) + "IHDR"(4) + width(4) + height(4)
    if data.len() < 24 {
        return None;
    }
    if &data[12..16] != b"IHDR" {
        return None;
    }
    let w = u32::from_be_bytes([data[16], data[17], data[18], data[19]]);
    let h = u32::from_be_bytes([data[20], data[21], data[22], data[23]]);
    Some((w, h))
}

fn detect_jpeg_dimensions(data: &[u8]) -> Option<(u32, u32)> {
    // Scan for SOFn markers (0xFFC0..0xFFC3) which contain dimensions.
    let mut i = 2; // skip SOI (0xFFD8)
    while i + 1 < data.len() {
        if data[i] != 0xFF {
            i += 1;
            continue;
        }
        let marker = data[i + 1];
        if marker == 0x00 || marker == 0xFF {
            i += 1;
            continue;
        }
        if (0xC0..=0xC3).contains(&marker) && i + 9 < data.len() {
            let h = u16::from_be_bytes([data[i + 5], data[i + 6]]) as u32;
            let w = u16::from_be_bytes([data[i + 7], data[i + 8]]) as u32;
            return Some((w, h));
        }
        if i + 3 < data.len() {
            let len = u16::from_be_bytes([data[i + 2], data[i + 3]]) as usize;
            i += 2 + len;
        } else {
            break;
        }
    }
    None
}

fn detect_bmp_dimensions(data: &[u8]) -> Option<(u32, u32)> {
    // BMP header: width at offset 18 (4 bytes LE), height at offset 22 (4 bytes LE, signed).
    if data.len() < 26 {
        return None;
    }
    let w = u32::from_le_bytes([data[18], data[19], data[20], data[21]]);
    let h_signed = i32::from_le_bytes([data[22], data[23], data[24], data[25]]);
    let h = h_signed.unsigned_abs();
    Some((w, h))
}

/// Internal multimedia resource definition used during assembly.
#[derive(Debug)]
pub(crate) struct MediaDef {
    pub id: u32,
    pub format: ImageFormat,
    /// File name within Res/ directory (e.g. "Image_0.jpg").
    pub file_name: String,
}
