//! High-level OFD writer / builder.

use std::io::{Cursor, Write};
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

use crate::model::document::{PageDef, PageSize};
use crate::model::graphic::{GraphicObject, ImageObject};
use crate::model::resource::{ImageFormat, MediaDef};
use crate::model::{DocInfo};
use crate::types::StBox;

use super::xml_gen;

/// Source for a single image to be embedded in the OFD.
pub struct ImageSource {
    pub bytes: Vec<u8>,
    pub format: ImageFormat,
    pub page_size: PageSize,
}

impl ImageSource {
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

    /// Create from image bytes with explicit format and mm dimensions.
    pub fn new(bytes: Vec<u8>, format: ImageFormat, width_mm: f64, height_mm: f64) -> Self {
        Self {
            bytes,
            format,
            page_size: PageSize::new(width_mm, height_mm),
        }
    }
}

/// OFD document writer.
///
/// # Examples
///
/// ```no_run
/// use ofd_rs::{OfdWriter, ImageSource};
///
/// let jpeg_bytes = std::fs::read("photo.jpg").unwrap();
/// let ofd_bytes = OfdWriter::from_images(vec![
///     ImageSource::jpeg(jpeg_bytes, 2480, 3508, 300.0),
/// ]).build().unwrap();
/// std::fs::write("output.ofd", ofd_bytes).unwrap();
/// ```
pub struct OfdWriter {
    doc_info: DocInfo,
    pages: Vec<PageEntry>,
}

struct PageEntry {
    image: ImageSource,
}

impl OfdWriter {
    /// Create a new OFD writer with default doc info.
    pub fn new() -> Self {
        Self {
            doc_info: DocInfo {
                doc_id: uuid::Uuid::new_v4().to_string().replace("-", ""),
                creator: Some("ofd-rs".into()),
                ..Default::default()
            },
            pages: Vec::new(),
        }
    }

    /// Quick constructor: one image per page.
    pub fn from_images(images: Vec<ImageSource>) -> Self {
        let mut writer = Self::new();
        for img in images {
            writer.pages.push(PageEntry { image: img });
        }
        writer
    }

    /// Set document metadata.
    pub fn set_doc_info(&mut self, info: DocInfo) -> &mut Self {
        self.doc_info = info;
        self
    }

    /// Add a page with an image.
    pub fn add_image_page(&mut self, image: ImageSource) -> &mut Self {
        self.pages.push(PageEntry { image });
        self
    }

    /// Build the OFD file and return the bytes.
    pub fn build(self) -> Result<Vec<u8>, OfdError> {
        if self.pages.is_empty() {
            return Err(OfdError::NoPages);
        }

        let mut id_counter: u32 = 0;
        let mut next_id = || -> u32 {
            id_counter += 1;
            id_counter
        };

        // Use the first page's size as the default page area
        let default_page_size = &self.pages[0].image.page_size;
        let default_page_area = default_page_size.to_box();

        // Prepare per-page data
        let mut page_defs: Vec<PageDef> = Vec::new();
        let mut media_defs: Vec<MediaDef> = Vec::new();
        // Store: (page_index, layer_id, image_object, page_area_override, image_bytes, image_file_path)
        struct PageBuildData {
            page_dir: String,
            layer_id: u32,
            objects: Vec<GraphicObject>,
            page_area: Option<StBox>,
            image_file_path: String,
            image_bytes: Vec<u8>,
        }
        let mut page_builds: Vec<PageBuildData> = Vec::new();

        for (i, entry) in self.pages.iter().enumerate() {
            let page_id = next_id();
            let page_dir = format!("Pages/Page_{}", i);

            page_defs.push(PageDef {
                page_id,
                base_loc: format!("{}/Content.xml", page_dir),
            });

            // Media resource
            let media_id = next_id();
            let file_name = format!("Image_{}.{}", i, entry.image.format.extension());
            media_defs.push(MediaDef {
                id: media_id,
                format: entry.image.format,
                file_name: file_name.clone(),
            });

            // Layer + ImageObject
            let layer_id = next_id();
            let img_obj_id = next_id();
            let page_size = &entry.image.page_size;
            let boundary = page_size.to_box();

            let objects = vec![GraphicObject::Image(ImageObject {
                id: img_obj_id,
                boundary,
                resource_id: media_id,
                alpha: None,
            })];

            // Only set per-page area if different from default
            let page_area_override = if (page_size.width_mm - default_page_size.width_mm).abs() > 0.01
                || (page_size.height_mm - default_page_size.height_mm).abs() > 0.01
            {
                Some(page_size.to_box())
            } else {
                None
            };

            let image_file_path = format!("Doc_0/Res/{}", file_name);

            page_builds.push(PageBuildData {
                page_dir,
                layer_id,
                objects,
                page_area: page_area_override,
                image_file_path,
                image_bytes: entry.image.bytes.clone(),
            });
        }

        let max_id = id_counter;

        // Generate XML content
        let ofd_xml = xml_gen::gen_ofd_xml(&self.doc_info);
        let document_xml = xml_gen::gen_document_xml(&page_defs, max_id, &default_page_area);
        let doc_res_xml = xml_gen::gen_document_res_xml(&media_defs);
        let pub_res_xml = xml_gen::gen_public_res_xml(&[]);

        // Build ZIP
        let buf = Cursor::new(Vec::new());
        let mut zip = ZipWriter::new(buf);
        let options = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);

        // OFD.xml
        zip.start_file("OFD.xml", options)
            .map_err(|e| OfdError::Zip(e.to_string()))?;
        zip.write_all(ofd_xml.as_bytes())
            .map_err(|e| OfdError::Zip(e.to_string()))?;

        // Doc_0/Document.xml
        zip.start_file("Doc_0/Document.xml", options)
            .map_err(|e| OfdError::Zip(e.to_string()))?;
        zip.write_all(document_xml.as_bytes())
            .map_err(|e| OfdError::Zip(e.to_string()))?;

        // Doc_0/DocumentRes.xml
        zip.start_file("Doc_0/DocumentRes.xml", options)
            .map_err(|e| OfdError::Zip(e.to_string()))?;
        zip.write_all(doc_res_xml.as_bytes())
            .map_err(|e| OfdError::Zip(e.to_string()))?;

        // Doc_0/PublicRes.xml
        zip.start_file("Doc_0/PublicRes.xml", options)
            .map_err(|e| OfdError::Zip(e.to_string()))?;
        zip.write_all(pub_res_xml.as_bytes())
            .map_err(|e| OfdError::Zip(e.to_string()))?;

        // Per-page Content.xml + image files
        for pb in &page_builds {
            let content_xml = xml_gen::gen_content_xml(
                pb.layer_id,
                &pb.objects,
                pb.page_area.as_ref(),
            );

            let content_path = format!("Doc_0/{}/Content.xml", pb.page_dir);
            zip.start_file(&content_path, options)
                .map_err(|e| OfdError::Zip(e.to_string()))?;
            zip.write_all(content_xml.as_bytes())
                .map_err(|e| OfdError::Zip(e.to_string()))?;

            // Image file
            zip.start_file(&pb.image_file_path, options)
                .map_err(|e| OfdError::Zip(e.to_string()))?;
            zip.write_all(&pb.image_bytes)
                .map_err(|e| OfdError::Zip(e.to_string()))?;
        }

        let result = zip.finish()
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
