//! XML generation for OFD document files.
//!
//! Each function produces the XML content for one file in the OFD package.
//! Structure follows ofdrw (https://github.com/ofdrw/ofdrw) conventions.

use crate::model::document::PageDef;
use crate::model::graphic::{GraphicObject, ImageObject};
use crate::model::ofd::DocInfo;
use crate::model::resource::MediaDef;
use crate::types::{format_mm_value, OFD_NAMESPACE, StBox};

/// OFD spec version. ofdrw uses "1.2".
const OFD_VERSION: &str = "1.2";

/// UTF-8 BOM + XML declaration.
const XML_HEADER: &str = "\u{FEFF}<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n";

/// Generate OFD.xml — the main entry file.
pub(crate) fn gen_ofd_xml(doc_info: &DocInfo) -> String {
    let mut xml = String::with_capacity(512);
    xml.push_str(XML_HEADER);
    xml.push_str(&format!(
        "<ofd:OFD xmlns:ofd=\"{}\" Version=\"{}\" DocType=\"OFD\">\n",
        OFD_NAMESPACE, OFD_VERSION
    ));
    xml.push_str("  <ofd:DocBody>\n");
    xml.push_str("    <ofd:DocInfo>\n");
    xml.push_str(&format!(
        "      <ofd:DocID>{}</ofd:DocID>\n",
        escape_xml(&doc_info.doc_id)
    ));
    if let Some(ref title) = doc_info.title {
        xml.push_str(&format!(
            "      <ofd:Title>{}</ofd:Title>\n",
            escape_xml(title)
        ));
    }
    if let Some(ref author) = doc_info.author {
        xml.push_str(&format!(
            "      <ofd:Author>{}</ofd:Author>\n",
            escape_xml(author)
        ));
    }
    if let Some(ref date) = doc_info.creation_date {
        xml.push_str(&format!(
            "      <ofd:CreationDate>{}</ofd:CreationDate>\n",
            escape_xml(date)
        ));
    }
    if let Some(ref creator) = doc_info.creator {
        xml.push_str(&format!(
            "      <ofd:Creator>{}</ofd:Creator>\n",
            escape_xml(creator)
        ));
    }
    if let Some(ref version) = doc_info.creator_version {
        xml.push_str(&format!(
            "      <ofd:CreatorVersion>{}</ofd:CreatorVersion>\n",
            escape_xml(version)
        ));
    }
    xml.push_str("    </ofd:DocInfo>\n");
    xml.push_str("    <ofd:DocRoot>Doc_0/Document.xml</ofd:DocRoot>\n");
    xml.push_str("  </ofd:DocBody>\n");
    xml.push_str("</ofd:OFD>\n");
    xml
}

/// Generate Document.xml — the document root.
pub(crate) fn gen_document_xml(
    pages: &[PageDef],
    max_id: u32,
    page_area: &StBox,
) -> String {
    let mut xml = String::with_capacity(1024);
    xml.push_str(XML_HEADER);
    xml.push_str(&format!(
        "<ofd:Document xmlns:ofd=\"{}\">\n",
        OFD_NAMESPACE
    ));

    // CommonData first (matches ofdrw order)
    xml.push_str("  <ofd:CommonData>\n");
    xml.push_str(&format!(
        "    <ofd:MaxUnitID>{}</ofd:MaxUnitID>\n",
        max_id
    ));
    xml.push_str("    <ofd:PageArea>\n");
    xml.push_str(&format!(
        "      <ofd:PhysicalBox>{}</ofd:PhysicalBox>\n",
        page_area
    ));
    xml.push_str(&format!(
        "      <ofd:ApplicationBox>{}</ofd:ApplicationBox>\n",
        page_area
    ));
    xml.push_str("    </ofd:PageArea>\n");
    xml.push_str("    <ofd:DocumentRes>DocumentRes.xml</ofd:DocumentRes>\n");
    xml.push_str("  </ofd:CommonData>\n");

    // Pages
    xml.push_str("  <ofd:Pages>\n");
    for page in pages {
        xml.push_str(&format!(
            "    <ofd:Page ID=\"{}\" BaseLoc=\"{}\"/>\n",
            page.page_id, page.base_loc
        ));
    }
    xml.push_str("  </ofd:Pages>\n");

    xml.push_str("</ofd:Document>\n");
    xml
}

/// Generate DocumentRes.xml — multimedia resource declarations.
pub(crate) fn gen_document_res_xml(medias: &[MediaDef]) -> String {
    let mut xml = String::with_capacity(512);
    xml.push_str(XML_HEADER);
    xml.push_str(&format!(
        "<ofd:Res xmlns:ofd=\"{}\" BaseLoc=\"Res\">\n",
        OFD_NAMESPACE
    ));

    if !medias.is_empty() {
        xml.push_str("  <ofd:MultiMedias>\n");
        for m in medias {
            xml.push_str(&format!(
                "    <ofd:MultiMedia ID=\"{}\" Type=\"Image\" Format=\"{}\">\n",
                m.id,
                m.format.ofd_format()
            ));
            xml.push_str(&format!(
                "      <ofd:MediaFile>{}</ofd:MediaFile>\n",
                m.file_name
            ));
            xml.push_str("    </ofd:MultiMedia>\n");
        }
        xml.push_str("  </ofd:MultiMedias>\n");
    }

    xml.push_str("</ofd:Res>\n");
    xml
}

/// Generate Content.xml — page content with layers and graphic objects.
pub(crate) fn gen_content_xml(
    layer_id: u32,
    objects: &[GraphicObject],
    page_area: Option<&StBox>,
) -> String {
    let mut xml = String::with_capacity(1024);
    xml.push_str(XML_HEADER);
    xml.push_str(&format!(
        "<ofd:Page xmlns:ofd=\"{}\">\n",
        OFD_NAMESPACE
    ));

    // Optional per-page area override (with both PhysicalBox and ApplicationBox)
    if let Some(area) = page_area {
        xml.push_str("  <ofd:Area>\n");
        xml.push_str(&format!(
            "    <ofd:PhysicalBox>{}</ofd:PhysicalBox>\n",
            area
        ));
        xml.push_str(&format!(
            "    <ofd:ApplicationBox>{}</ofd:ApplicationBox>\n",
            area
        ));
        xml.push_str("  </ofd:Area>\n");
    }

    // Content without ID attribute (matches ofdrw)
    xml.push_str("  <ofd:Content>\n");
    xml.push_str(&format!(
        "    <ofd:Layer ID=\"{}\" Type=\"Body\">\n",
        layer_id
    ));

    for obj in objects {
        match obj {
            GraphicObject::Image(img) => {
                write_image_object(&mut xml, img, 6);
            }
        }
    }

    xml.push_str("    </ofd:Layer>\n");
    xml.push_str("  </ofd:Content>\n");
    xml.push_str("</ofd:Page>\n");
    xml
}

fn write_image_object(xml: &mut String, img: &ImageObject, indent: usize) {
    let pad: String = " ".repeat(indent);
    let ctm_w = format_mm_value(img.boundary.w);
    let ctm_h = format_mm_value(img.boundary.h);
    xml.push_str(&format!(
        "{pad}<ofd:ImageObject ID=\"{id}\" ResourceID=\"{res}\" \
         Boundary=\"{boundary}\" CTM=\"{cw} 0 0 {ch} 0 0\"",
        pad = pad,
        id = img.id,
        res = img.resource_id,
        boundary = img.boundary,
        cw = ctm_w,
        ch = ctm_h,
    ));
    if let Some(alpha) = img.alpha {
        xml.push_str(&format!(" Alpha=\"{}\"", alpha));
    }
    xml.push_str("/>\n");
}

/// Escape XML special characters in text content / attribute values.
fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
