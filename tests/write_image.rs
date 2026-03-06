//! Integration test: generate an OFD from a synthetic JPEG image.

use ofd_rs::{DocInfo, ImageSource, OfdWriter};
use std::io::{Cursor, Read};
use zip::ZipArchive;

/// Create a minimal valid JPEG (1x1 pixel, white).
fn tiny_jpeg() -> Vec<u8> {
    // Minimal JFIF JPEG: SOI + APP0 + DQT + SOF0 + DHT + SOS + data + EOI
    // This is a hand-crafted minimal JPEG that decoders accept.
    vec![
        0xFF, 0xD8, // SOI
        0xFF, 0xE0, // APP0
        0x00, 0x10, // length 16
        0x4A, 0x46, 0x49, 0x46, 0x00, // "JFIF\0"
        0x01, 0x01, // version 1.1
        0x00, // no units
        0x00, 0x01, // X density 1
        0x00, 0x01, // Y density 1
        0x00, 0x00, // no thumbnail
        0xFF, 0xDB, // DQT
        0x00, 0x43, // length 67
        0x00, // 8-bit, table 0
        // 64 quantization values (all 1 for simplicity)
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        0xFF, 0xC0, // SOF0 (baseline)
        0x00, 0x0B, // length 11
        0x08, // 8-bit precision
        0x00, 0x01, // height 1
        0x00, 0x01, // width 1
        0x01, // 1 component
        0x01, // component ID 1
        0x11, // sampling 1x1
        0x00, // quant table 0
        0xFF, 0xC4, // DHT
        0x00, 0x1F, // length 31
        0x00, // DC table 0
        // Counts for codes of length 1..16
        0x00, 0x01, 0x05, 0x01, 0x01, 0x01, 0x01, 0x01,
        0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        // Values
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B,
        0xFF, 0xC4, // DHT
        0x00, 0xB5, // length 181
        0x10, // AC table 0
        // Counts
        0x00, 0x02, 0x01, 0x03, 0x03, 0x02, 0x04, 0x03,
        0x05, 0x05, 0x04, 0x04, 0x00, 0x00, 0x01, 0x7D,
        // Values (standard AC table, 162 values)
        0x01, 0x02, 0x03, 0x00, 0x04, 0x11, 0x05, 0x12,
        0x21, 0x31, 0x41, 0x06, 0x13, 0x51, 0x61, 0x07,
        0x22, 0x71, 0x14, 0x32, 0x81, 0x91, 0xA1, 0x08,
        0x23, 0x42, 0xB1, 0xC1, 0x15, 0x52, 0xD1, 0xF0,
        0x24, 0x33, 0x62, 0x72, 0x82, 0x09, 0x0A, 0x16,
        0x17, 0x18, 0x19, 0x1A, 0x25, 0x26, 0x27, 0x28,
        0x29, 0x2A, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39,
        0x3A, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49,
        0x4A, 0x53, 0x54, 0x55, 0x56, 0x57, 0x58, 0x59,
        0x5A, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69,
        0x6A, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79,
        0x7A, 0x83, 0x84, 0x85, 0x86, 0x87, 0x88, 0x89,
        0x8A, 0x92, 0x93, 0x94, 0x95, 0x96, 0x97, 0x98,
        0x99, 0x9A, 0xA2, 0xA3, 0xA4, 0xA5, 0xA6, 0xA7,
        0xA8, 0xA9, 0xAA, 0xB2, 0xB3, 0xB4, 0xB5, 0xB6,
        0xB7, 0xB8, 0xB9, 0xBA, 0xC2, 0xC3, 0xC4, 0xC5,
        0xC6, 0xC7, 0xC8, 0xC9, 0xCA, 0xD2, 0xD3, 0xD4,
        0xD5, 0xD6, 0xD7, 0xD8, 0xD9, 0xDA, 0xE1, 0xE2,
        0xE3, 0xE4, 0xE5, 0xE6, 0xE7, 0xE8, 0xE9, 0xEA,
        0xF1, 0xF2, 0xF3, 0xF4, 0xF5, 0xF6, 0xF7, 0xF8,
        0xF9, 0xFA,
        0xFF, 0xDA, // SOS
        0x00, 0x08, // length 8
        0x01, // 1 component
        0x01, // component 1
        0x00, // DC/AC table 0/0
        0x00, 0x3F, 0x00, // spectral selection
        0x7B, 0x40, // compressed data (white pixel)
        0xFF, 0xD9, // EOI
    ]
}

#[test]
fn generate_single_page_ofd() {
    let jpeg = tiny_jpeg();
    let ofd_bytes = OfdWriter::from_images(vec![
        ImageSource::jpeg_mm(jpeg.clone(), 210.0, 297.0),
    ])
    .build()
    .expect("build should succeed");

    // Verify it's a valid ZIP
    let cursor = Cursor::new(&ofd_bytes);
    let mut archive = ZipArchive::new(cursor).expect("should be valid ZIP");

    // Check required files exist
    let expected_files = [
        "OFD.xml",
        "Doc_0/Document.xml",
        "Doc_0/DocumentRes.xml",
        "Doc_0/PublicRes.xml",
        "Doc_0/Pages/Page_0/Content.xml",
        "Doc_0/Res/Image_0.jpg",
    ];

    for name in &expected_files {
        assert!(
            archive.by_name(name).is_ok(),
            "missing file in OFD: {}",
            name
        );
    }

    // Verify OFD.xml content
    let mut ofd_xml = String::new();
    archive
        .by_name("OFD.xml")
        .unwrap()
        .read_to_string(&mut ofd_xml)
        .unwrap();
    assert!(ofd_xml.contains("ofd:OFD"));
    assert!(ofd_xml.contains("xmlns:ofd=\"http://www.ofdspec.org/2016\""));
    assert!(ofd_xml.contains("Version=\"1.1\""));
    assert!(ofd_xml.contains("DocType=\"OFD\""));
    assert!(ofd_xml.contains("ofd:DocRoot"));
    assert!(ofd_xml.contains("ofd:DocID"));

    // Verify Document.xml
    let mut doc_xml = String::new();
    archive
        .by_name("Doc_0/Document.xml")
        .unwrap()
        .read_to_string(&mut doc_xml)
        .unwrap();
    assert!(doc_xml.contains("ofd:Document"));
    assert!(doc_xml.contains("ofd:PhysicalBox"));
    assert!(doc_xml.contains("ofd:Page"));
    assert!(doc_xml.contains("ofd:PublicRes"));
    assert!(doc_xml.contains("ofd:DocumentRes"));

    // Verify Content.xml has ImageObject
    let mut content_xml = String::new();
    archive
        .by_name("Doc_0/Pages/Page_0/Content.xml")
        .unwrap()
        .read_to_string(&mut content_xml)
        .unwrap();
    assert!(content_xml.contains("ofd:ImageObject"));
    assert!(content_xml.contains("Boundary="));
    assert!(content_xml.contains("ResourceID="));

    // Verify image data is preserved
    let mut image_data = Vec::new();
    archive
        .by_name("Doc_0/Res/Image_0.jpg")
        .unwrap()
        .read_to_end(&mut image_data)
        .unwrap();
    assert_eq!(image_data, jpeg);
}

#[test]
fn generate_multi_page_ofd() {
    let jpeg = tiny_jpeg();
    let ofd_bytes = OfdWriter::from_images(vec![
        ImageSource::jpeg_mm(jpeg.clone(), 210.0, 297.0),
        ImageSource::jpeg_mm(jpeg.clone(), 210.0, 297.0),
        ImageSource::jpeg_mm(jpeg.clone(), 100.0, 150.0), // different size
    ])
    .build()
    .expect("build should succeed");

    let cursor = Cursor::new(&ofd_bytes);
    let archive = ZipArchive::new(cursor).expect("should be valid ZIP");

    // Should have 3 page content files and 3 image files
    for i in 0..3 {
        let content = format!("Doc_0/Pages/Page_{}/Content.xml", i);
        let image = format!("Doc_0/Res/Image_{}.jpg", i);
        assert!(archive.index_for_name(&content).is_some(), "missing {}", content);
        assert!(archive.index_for_name(&image).is_some(), "missing {}", image);
    }
}

#[test]
fn custom_doc_info() {
    let jpeg = tiny_jpeg();
    let mut writer = OfdWriter::new();
    writer.set_doc_info(DocInfo {
        doc_id: "abc123".into(),
        title: Some("Test Document".into()),
        author: Some("Test Author".into()),
        creator: Some("ofd-rs-test".into()),
        creator_version: Some("0.1.0".into()),
        creation_date: Some("2026-03-06".into()),
    });
    writer.add_image_page(ImageSource::jpeg_mm(jpeg, 210.0, 297.0));

    let ofd_bytes = writer.build().expect("build should succeed");
    let cursor = Cursor::new(&ofd_bytes);
    let mut archive = ZipArchive::new(cursor).unwrap();

    let mut ofd_xml = String::new();
    archive
        .by_name("OFD.xml")
        .unwrap()
        .read_to_string(&mut ofd_xml)
        .unwrap();

    assert!(ofd_xml.contains("<ofd:DocID>abc123</ofd:DocID>"));
    assert!(ofd_xml.contains("<ofd:Title>Test Document</ofd:Title>"));
    assert!(ofd_xml.contains("<ofd:Author>Test Author</ofd:Author>"));
    assert!(ofd_xml.contains("<ofd:Creator>ofd-rs-test</ofd:Creator>"));
    assert!(ofd_xml.contains("<ofd:CreationDate>2026-03-06</ofd:CreationDate>"));
}

#[test]
fn empty_pages_returns_error() {
    let writer = OfdWriter::new();
    let result = writer.build();
    assert!(result.is_err());
}
