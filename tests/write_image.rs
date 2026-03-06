//! Integration tests: generate OFD from synthetic images and verify structure.

use ofd_rs::{DocInfo, ImageFormat, ImageSource, OfdWriter};
use std::io::{Cursor, Read};
use zip::ZipArchive;

/// Create a minimal valid JPEG (1x1 pixel, white).
fn tiny_jpeg() -> Vec<u8> {
    vec![
        0xFF, 0xD8, // SOI
        0xFF, 0xE0, // APP0
        0x00, 0x10, // length 16
        0x4A, 0x46, 0x49, 0x46, 0x00, // "JFIF\0"
        0x01, 0x01, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00,
        0xFF, 0xDB, // DQT
        0x00, 0x43, 0x00,
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        0xFF, 0xC0, // SOF0
        0x00, 0x0B, 0x08, 0x00, 0x01, 0x00, 0x01, 0x01, 0x01, 0x11, 0x00,
        0xFF, 0xC4, // DHT (DC)
        0x00, 0x1F, 0x00,
        0x00, 0x01, 0x05, 0x01, 0x01, 0x01, 0x01, 0x01,
        0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B,
        0xFF, 0xC4, // DHT (AC)
        0x00, 0xB5, 0x10,
        0x00, 0x02, 0x01, 0x03, 0x03, 0x02, 0x04, 0x03,
        0x05, 0x05, 0x04, 0x04, 0x00, 0x00, 0x01, 0x7D,
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
        0x00, 0x08, 0x01, 0x01, 0x00, 0x00, 0x3F, 0x00,
        0x7B, 0x40,
        0xFF, 0xD9, // EOI
    ]
}

/// Create a minimal PNG (1x1 pixel).
fn tiny_png() -> Vec<u8> {
    vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
        // IHDR
        0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
        0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01,
        0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77, 0x53, 0xDE,
        // IDAT
        0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, 0x54,
        0x08, 0xD7, 0x63, 0xF8, 0xCF, 0xC0, 0x00, 0x00,
        0x01, 0x01, 0x01, 0x00,
        // IEND
        0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44,
        0xAE, 0x42, 0x60, 0x82,
    ]
}

fn build_and_open(images: Vec<ImageSource>) -> ZipArchive<Cursor<Vec<u8>>> {
    let ofd_bytes = OfdWriter::from_images(images)
        .build()
        .expect("build should succeed");
    ZipArchive::new(Cursor::new(ofd_bytes)).expect("should be valid ZIP")
}

fn read_zip_string(archive: &mut ZipArchive<Cursor<Vec<u8>>>, name: &str) -> String {
    let mut s = String::new();
    archive
        .by_name(name)
        .unwrap_or_else(|_| panic!("missing file: {}", name))
        .read_to_string(&mut s)
        .unwrap();
    s
}

#[test]
fn generate_single_page_ofd() {
    let jpeg = tiny_jpeg();
    let mut archive = build_and_open(vec![
        ImageSource::jpeg_mm(jpeg.clone(), 210.0, 297.0),
    ]);

    // No PublicRes.xml (matching ofdrw for image-only documents)
    let expected_files = [
        "OFD.xml",
        "Doc_0/Document.xml",
        "Doc_0/DocumentRes.xml",
        "Doc_0/Pages/Page_0/Content.xml",
        "Doc_0/Res/Image_0.jpg",
    ];
    for name in &expected_files {
        assert!(archive.by_name(name).is_ok(), "missing file: {}", name);
    }
    assert!(
        archive.by_name("Doc_0/PublicRes.xml").is_err(),
        "PublicRes.xml should NOT exist for image-only OFD"
    );

    // OFD.xml
    let ofd_xml = read_zip_string(&mut archive, "OFD.xml");
    assert!(ofd_xml.contains("xmlns:ofd=\"http://www.ofdspec.org/2016\""));
    assert!(ofd_xml.contains("Version=\"1.2\""));
    assert!(ofd_xml.contains("DocType=\"OFD\""));
    assert!(ofd_xml.contains("<ofd:Creator>ofd-rs</ofd:Creator>"));
    assert!(ofd_xml.contains("<ofd:CreatorVersion>"));
    assert!(ofd_xml.contains("<ofd:CreationDate>"));

    // Document.xml — CommonData before Pages, ApplicationBox present, no PublicRes
    let doc_xml = read_zip_string(&mut archive, "Doc_0/Document.xml");
    assert!(doc_xml.contains("ofd:DocumentRes"));
    assert!(!doc_xml.contains("ofd:PublicRes"));
    assert!(doc_xml.contains("ofd:PhysicalBox"));
    assert!(doc_xml.contains("ofd:ApplicationBox"));
    let common_pos = doc_xml.find("ofd:CommonData").unwrap();
    let pages_pos = doc_xml.find("ofd:Pages").unwrap();
    assert!(common_pos < pages_pos, "CommonData should come before Pages");

    // Content.xml — no ID on Content, Layer has ID and Type
    let content_xml = read_zip_string(&mut archive, "Doc_0/Pages/Page_0/Content.xml");
    assert!(content_xml.contains("<ofd:Content>"), "Content should have no ID attribute");
    assert!(content_xml.contains("ofd:Layer ID="));
    assert!(content_xml.contains("Type=\"Body\""));
    assert!(content_xml.contains("ofd:ImageObject"));
    assert!(content_xml.contains("CTM="));
    assert!(content_xml.contains("ResourceID="));

    // DocumentRes.xml — ID before Type
    let res_xml = read_zip_string(&mut archive, "Doc_0/DocumentRes.xml");
    assert!(res_xml.contains("Format=\"JPEG\""));
    let id_pos = res_xml.find("ID=").unwrap();
    let type_pos = res_xml.find("Type=").unwrap();
    assert!(id_pos < type_pos, "MultiMedia: ID should come before Type");

    // Image data preserved
    let mut image_data = Vec::new();
    archive
        .by_name("Doc_0/Res/Image_0.jpg")
        .unwrap()
        .read_to_end(&mut image_data)
        .unwrap();
    assert_eq!(image_data, jpeg);
}

#[test]
fn generate_png_page_ofd() {
    let png = tiny_png();
    let mut archive = build_and_open(vec![
        ImageSource::png_mm(png.clone(), 210.0, 297.0),
    ]);

    assert!(
        archive.by_name("Doc_0/Res/Image_0.png").is_ok(),
        "missing PNG file"
    );

    let res_xml = read_zip_string(&mut archive, "Doc_0/DocumentRes.xml");
    assert!(res_xml.contains("Image_0.png"));
    assert!(res_xml.contains("Format=\"PNG\""));
}

#[test]
fn auto_detect_format() {
    let jpeg = tiny_jpeg();
    let png = tiny_png();

    let src_jpeg = ImageSource::auto_detect(jpeg, 150.0).expect("should detect JPEG");
    assert_eq!(src_jpeg.format, ImageFormat::Jpeg);
    assert!(src_jpeg.page_size.width_mm > 0.0);

    let src_png = ImageSource::auto_detect(png, 150.0).expect("should detect PNG");
    assert_eq!(src_png.format, ImageFormat::Png);
    assert!(src_png.page_size.width_mm > 0.0);

    assert!(ImageSource::auto_detect(vec![0x00, 0x01, 0x02, 0x03], 150.0).is_none());

    // auto_detect_default uses ofdrw's 5 px/mm ratio: 1px / 5 = 0.2 mm
    let src_default = ImageSource::auto_detect_default(tiny_jpeg()).expect("should detect JPEG");
    assert!((src_default.page_size.width_mm - 0.2).abs() < 0.001);
    assert!((src_default.page_size.height_mm - 0.2).abs() < 0.001);
}

#[test]
fn generate_multi_page_ofd() {
    let jpeg = tiny_jpeg();
    let archive = build_and_open(vec![
        ImageSource::jpeg_mm(jpeg.clone(), 210.0, 297.0),
        ImageSource::jpeg_mm(jpeg.clone(), 210.0, 297.0),
        ImageSource::jpeg_mm(jpeg.clone(), 100.0, 150.0),
    ]);

    for i in 0..3 {
        let content = format!("Doc_0/Pages/Page_{}/Content.xml", i);
        let image = format!("Doc_0/Res/Image_{}.jpg", i);
        assert!(archive.index_for_name(&content).is_some(), "missing {}", content);
        assert!(archive.index_for_name(&image).is_some(), "missing {}", image);
    }
}

#[test]
fn multi_page_different_size_has_area_override() {
    let jpeg = tiny_jpeg();
    let mut archive = build_and_open(vec![
        ImageSource::jpeg_mm(jpeg.clone(), 210.0, 297.0),
        ImageSource::jpeg_mm(jpeg.clone(), 100.0, 150.0),
    ]);

    // Page 0 uses default area — no <ofd:Area> in its Content.xml
    let page0 = read_zip_string(&mut archive, "Doc_0/Pages/Page_0/Content.xml");
    assert!(!page0.contains("ofd:Area"), "Page 0 should use default area");

    // Page 1 has different size — should have <ofd:Area> with both boxes
    let page1 = read_zip_string(&mut archive, "Doc_0/Pages/Page_1/Content.xml");
    assert!(page1.contains("ofd:Area"), "Page 1 should have area override");
    assert!(page1.contains("ofd:PhysicalBox"));
    assert!(page1.contains("ofd:ApplicationBox"));
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
    let mut archive = ZipArchive::new(Cursor::new(ofd_bytes)).unwrap();

    let ofd_xml = read_zip_string(&mut archive, "OFD.xml");
    assert!(ofd_xml.contains("<ofd:DocID>abc123</ofd:DocID>"));
    assert!(ofd_xml.contains("<ofd:Title>Test Document</ofd:Title>"));
    assert!(ofd_xml.contains("<ofd:Author>Test Author</ofd:Author>"));
    assert!(ofd_xml.contains("<ofd:Creator>ofd-rs-test</ofd:Creator>"));
    assert!(ofd_xml.contains("<ofd:CreatorVersion>0.1.0</ofd:CreatorVersion>"));
    assert!(ofd_xml.contains("<ofd:CreationDate>2026-03-06</ofd:CreationDate>"));
}

#[test]
fn default_doc_info_has_metadata() {
    let jpeg = tiny_jpeg();
    let mut archive = build_and_open(vec![
        ImageSource::jpeg_mm(jpeg, 210.0, 297.0),
    ]);

    let ofd_xml = read_zip_string(&mut archive, "OFD.xml");
    assert!(ofd_xml.contains("<ofd:Creator>ofd-rs</ofd:Creator>"));
    assert!(ofd_xml.contains("<ofd:CreatorVersion>"));
    assert!(ofd_xml.contains("<ofd:CreationDate>"));
}

#[test]
fn empty_pages_returns_error() {
    assert!(OfdWriter::new().build().is_err());
}
