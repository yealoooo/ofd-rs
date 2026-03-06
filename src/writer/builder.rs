//! High-level OFD writer / builder.

use std::io::{Cursor, Write};
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

use crate::model::document::{PageDef, PageSize, PPM_DEFAULT};
use crate::model::graphic::{GraphicObject, ImageObject};
use crate::model::ofd::DocInfo;
use crate::model::resource::{detect_image_dimensions, ImageFormat, MediaDef};
use crate::types::StBox;

use super::xml_gen;

/// Library name used as default Creator in DocInfo.
const LIB_NAME: &str = "ofd-rs";

/// Library version used as default CreatorVersion in DocInfo.
const LIB_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Source for a single image page to be embedded in the OFD.
///
/// Each `ImageSource` becomes one page in the output document.
/// The image fills the entire page boundary (no margins).
pub struct ImageSource {
    /// Raw image bytes.
    pub bytes: Vec<u8>,
    /// Detected or specified image format.
    pub format: ImageFormat,
    /// Page size in millimeters.
    pub page_size: PageSize,
}

impl ImageSource {
    /// Create from image bytes with explicit format and mm dimensions.
    pub fn new(bytes: Vec<u8>, format: ImageFormat, width_mm: f64, height_mm: f64) -> Self {
        Self {
            bytes,
            format,
            page_size: PageSize::new(width_mm, height_mm),
        }
    }

    /// Create from JPEG bytes with known pixel dimensions and DPI.
    pub fn jpeg(bytes: Vec<u8>, width_px: u32, height_px: u32, dpi: f64) -> Self {
        Self {
            bytes,
            format: ImageFormat::Jpeg,
            page_size: PageSize::from_pixels(width_px, height_px, dpi),
        }
    }

    /// Create from JPEG bytes with page size in mm.
    pub fn jpeg_mm(bytes: Vec<u8>, width_mm: f64, height_mm: f64) -> Self {
        Self {
            bytes,
            format: ImageFormat::Jpeg,
            page_size: PageSize::new(width_mm, height_mm),
        }
    }

    /// Create from PNG bytes with page size in mm.
    pub fn png_mm(bytes: Vec<u8>, width_mm: f64, height_mm: f64) -> Self {
        Self {
            bytes,
            format: ImageFormat::Png,
            page_size: PageSize::new(width_mm, height_mm),
        }
    }

    /// Auto-detect format and dimensions, calculate page size from DPI.
    ///
    /// Returns `None` if format or dimensions cannot be detected.
    pub fn auto_detect(bytes: Vec<u8>, dpi: f64) -> Option<Self> {
        let format = ImageFormat::detect(&bytes)?;
        let (w_px, h_px) = detect_image_dimensions(&bytes)?;
        Some(Self {
            bytes,
            format,
            page_size: PageSize::from_pixels(w_px, h_px, dpi),
        })
    }

    /// Auto-detect format and dimensions using pixels-per-mm ratio.
    ///
    /// Uses ofdrw's conversion: `mm = pixels / ppm`.
    /// See [`PPM_DEFAULT`] for the standard 5 px/mm ratio.
    /// Returns `None` if format or dimensions cannot be detected.
    pub fn auto_detect_ppm(bytes: Vec<u8>, ppm: f64) -> Option<Self> {
        let format = ImageFormat::detect(&bytes)?;
        let (w_px, h_px) = detect_image_dimensions(&bytes)?;
        Some(Self {
            bytes,
            format,
            page_size: PageSize::from_pixels_ppm(w_px, h_px, ppm),
        })
    }

    /// Auto-detect format and dimensions using ofdrw's default ratio (5 px/mm).
    ///
    /// Equivalent to `auto_detect_ppm(bytes, PPM_DEFAULT)`.
    /// Returns `None` if format or dimensions cannot be detected.
    pub fn auto_detect_default(bytes: Vec<u8>) -> Option<Self> {
        Self::auto_detect_ppm(bytes, PPM_DEFAULT)
    }

    /// Auto-detect format, use explicit mm dimensions for page size.
    ///
    /// Returns `None` if format cannot be detected from file header.
    pub fn auto_detect_mm(bytes: Vec<u8>, width_mm: f64, height_mm: f64) -> Option<Self> {
        let format = ImageFormat::detect(&bytes)?;
        Some(Self {
            bytes,
            format,
            page_size: PageSize::new(width_mm, height_mm),
        })
    }
}

/// OFD document writer.
///
/// Generates an OFD file (ZIP archive) containing image pages.
///
/// # Examples
///
/// ```no_run
/// use ofd_rs::{OfdWriter, ImageSource};
///
/// let jpeg_bytes = std::fs::read("photo.jpg").unwrap();
/// let ofd = OfdWriter::from_images(vec![
///     ImageSource::jpeg(jpeg_bytes, 2480, 3508, 300.0),
/// ]).build().unwrap();
/// std::fs::write("output.ofd", ofd).unwrap();
/// ```
pub struct OfdWriter {
    doc_info: DocInfo,
    pages: Vec<ImageSource>,
}

impl OfdWriter {
    /// Create a new writer with default metadata (matching ofdrw conventions).
    pub fn new() -> Self {
        Self {
            doc_info: DocInfo {
                doc_id: uuid::Uuid::new_v4().to_string().replace('-', ""),
                creator: Some(LIB_NAME.into()),
                creator_version: Some(LIB_VERSION.into()),
                creation_date: Some(today()),
                ..Default::default()
            },
            pages: Vec::new(),
        }
    }

    /// Quick constructor: one image per page.
    pub fn from_images(images: Vec<ImageSource>) -> Self {
        let mut writer = Self::new();
        writer.pages = images;
        writer
    }

    /// Set document metadata.
    pub fn set_doc_info(&mut self, info: DocInfo) -> &mut Self {
        self.doc_info = info;
        self
    }

    /// Add a page with an image.
    pub fn add_image_page(&mut self, image: ImageSource) -> &mut Self {
        self.pages.push(image);
        self
    }

    /// Build the OFD file and return the ZIP bytes.
    pub fn build(self) -> Result<Vec<u8>, OfdError> {
        if self.pages.is_empty() {
            return Err(OfdError::NoPages);
        }

        let mut id_counter: u32 = 0;
        let mut next_id = || -> u32 {
            id_counter += 1;
            id_counter
        };

        // First page's size as default page area
        let default_page_size = &self.pages[0].page_size;
        let default_page_area = default_page_size.to_box();

        let mut page_defs: Vec<PageDef> = Vec::new();
        let mut media_defs: Vec<MediaDef> = Vec::new();

        struct PageBuildData {
            page_dir: String,
            layer_id: u32,
            objects: Vec<GraphicObject>,
            page_area: Option<StBox>,
            image_file_path: String,
            image_bytes: Vec<u8>,
        }
        let mut page_builds: Vec<PageBuildData> = Vec::new();

        for (i, source) in self.pages.iter().enumerate() {
            let page_id = next_id();
            let page_dir = format!("Pages/Page_{}", i);

            page_defs.push(PageDef {
                page_id,
                base_loc: format!("{}/Content.xml", page_dir),
            });

            // Media resource
            let media_id = next_id();
            let file_name = format!("Image_{}.{}", i, source.format.extension());
            media_defs.push(MediaDef {
                id: media_id,
                format: source.format,
                file_name: file_name.clone(),
            });

            // Layer + ImageObject
            let layer_id = next_id();
            let img_obj_id = next_id();
            let boundary = source.page_size.to_box();

            let objects = vec![GraphicObject::Image(ImageObject {
                id: img_obj_id,
                boundary,
                resource_id: media_id,
                alpha: None,
            })];

            // Per-page area override only if different from default
            let page_area = if differs(&source.page_size, default_page_size) {
                Some(source.page_size.to_box())
            } else {
                None
            };

            page_builds.push(PageBuildData {
                page_dir,
                layer_id,
                objects,
                page_area,
                image_file_path: format!("Doc_0/Res/{}", file_name),
                image_bytes: source.bytes.clone(),
            });
        }

        let max_id = id_counter;

        // Generate XML
        let ofd_xml = xml_gen::gen_ofd_xml(&self.doc_info);
        let document_xml = xml_gen::gen_document_xml(&page_defs, max_id, &default_page_area);
        let doc_res_xml = xml_gen::gen_document_res_xml(&media_defs);

        // Build ZIP
        let buf = Cursor::new(Vec::new());
        let mut zip = ZipWriter::new(buf);
        let options = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);

        zip_file(&mut zip, "OFD.xml", ofd_xml.as_bytes(), options)?;
        zip_file(&mut zip, "Doc_0/Document.xml", document_xml.as_bytes(), options)?;
        zip_file(&mut zip, "Doc_0/DocumentRes.xml", doc_res_xml.as_bytes(), options)?;

        for pb in &page_builds {
            let content_xml = xml_gen::gen_content_xml(
                pb.layer_id,
                &pb.objects,
                pb.page_area.as_ref(),
            );
            let content_path = format!("Doc_0/{}/Content.xml", pb.page_dir);
            zip_file(&mut zip, &content_path, content_xml.as_bytes(), options)?;
            zip_file(&mut zip, &pb.image_file_path, &pb.image_bytes, options)?;
        }

        let result = zip
            .finish()
            .map_err(|e| OfdError::Zip(e.to_string()))?;
        Ok(result.into_inner())
    }
}

impl Default for OfdWriter {
    fn default() -> Self {
        Self::new()
    }
}

/// Errors that can occur during OFD generation.
#[derive(Debug)]
pub enum OfdError {
    /// No pages were added to the writer.
    NoPages,
    /// ZIP packaging error.
    Zip(String),
}

impl std::fmt::Display for OfdError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoPages => write!(f, "no pages added to OFD writer"),
            Self::Zip(msg) => write!(f, "ZIP error: {}", msg),
        }
    }
}

impl std::error::Error for OfdError {}

// --- helpers ---

fn differs(a: &PageSize, b: &PageSize) -> bool {
    (a.width_mm - b.width_mm).abs() > 0.01 || (a.height_mm - b.height_mm).abs() > 0.01
}

fn zip_file(
    zip: &mut ZipWriter<Cursor<Vec<u8>>>,
    path: &str,
    data: &[u8],
    options: SimpleFileOptions,
) -> Result<(), OfdError> {
    zip.start_file(path, options)
        .map_err(|e| OfdError::Zip(e.to_string()))?;
    zip.write_all(data)
        .map_err(|e| OfdError::Zip(e.to_string()))?;
    Ok(())
}

fn today() -> String {
    // Use system time to get current date in yyyy-MM-dd format.
    let now = std::time::SystemTime::now();
    let secs = now
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    // Simple date calculation (no external crate needed)
    let days = (secs / 86400) as i64;
    let (y, m, d) = days_to_ymd(days);
    format!("{:04}-{:02}-{:02}", y, m, d)
}

fn days_to_ymd(days_since_epoch: i64) -> (i32, u32, u32) {
    // Algorithm from http://howardhinnant.github.io/date_algorithms.html
    let z = days_since_epoch + 719468;
    let era = (if z >= 0 { z } else { z - 146096 }) / 146097;
    let doe = (z - era * 146097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = (yoe as i64 + era * 400) as i32;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}
