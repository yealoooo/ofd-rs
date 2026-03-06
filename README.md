# ofd-rs

Rust library for creating OFD (Open Fixed-layout Document) files, based on the Chinese national standard **GB/T 33190-2016**.

[![Crates.io](https://img.shields.io/crates/v/ofd-rs.svg)](https://crates.io/crates/ofd-rs)
[![Documentation](https://docs.rs/ofd-rs/badge.svg)](https://docs.rs/ofd-rs)
[![License](https://img.shields.io/crates/l/ofd-rs.svg)](https://github.com/yealou/ofd-rs/blob/main/LICENSE)

## What is OFD?

OFD (Open Fixed-layout Document) is a fixed-layout document format standardized by the Chinese national standard GB/T 33190-2016. It serves a similar role to PDF and is widely used in Chinese government and enterprise document workflows, especially for electronic invoices, official documents, and archival purposes.

An OFD file is a ZIP archive containing XML descriptors and embedded resources (images, fonts, etc.).

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
ofd-rs = "0.1"
```

## Quick Start

```rust
use ofd_rs::{OfdWriter, ImageSource};

// Convert a JPEG image to a single-page OFD document
let jpeg_bytes = std::fs::read("photo.jpg").unwrap();
let ofd = OfdWriter::from_images(vec![
    ImageSource::jpeg(jpeg_bytes, 2480, 3508, 300.0), // width_px, height_px, DPI
]).build().unwrap();
std::fs::write("output.ofd", ofd).unwrap();
```

## Features

### Current (v0.1)

- **Image to OFD** - Convert images (JPEG, PNG, BMP, TIFF) to OFD documents
- **Multi-page support** - Combine multiple images into a single OFD file
- **Mixed page sizes** - Each page can have different dimensions
- **Custom metadata** - Set document title, author, creator, creation date
- **Builder pattern** - Flexible API for constructing OFD documents
- **Spec compliant** - Follows GB/T 33190-2016 namespace and structure conventions
- **Minimal dependencies** - Only `zip` and `uuid`, no heavy XML frameworks

### API Overview

**Quick API** - One image per page:

```rust
use ofd_rs::{OfdWriter, ImageSource};

let ofd = OfdWriter::from_images(vec![
    ImageSource::jpeg(page1_bytes, 2480, 3508, 300.0),
    ImageSource::jpeg(page2_bytes, 2480, 3508, 300.0),
]).build()?;
```

**Builder API** - More control:

```rust
use ofd_rs::{OfdWriter, ImageSource, DocInfo};

let mut writer = OfdWriter::new();
writer.set_doc_info(DocInfo {
    doc_id: "my-doc-id".into(),
    title: Some("Invoice".into()),
    author: Some("Company".into()),
    creator: Some("MyApp".into()),
    creator_version: Some("1.0".into()),
    creation_date: Some("2026-03-06".into()),
});
writer.add_image_page(ImageSource::jpeg_mm(jpeg_bytes, 210.0, 297.0)); // A4 size in mm
let ofd = writer.build()?;
```

**Page size options**:

```rust
// From pixel dimensions + DPI (auto-calculates mm)
ImageSource::jpeg(bytes, width_px, height_px, dpi);

// From explicit mm dimensions
ImageSource::jpeg_mm(bytes, width_mm, height_mm);

// Any format with explicit mm dimensions
ImageSource::new(bytes, ImageFormat::Png, width_mm, height_mm);
```

## OFD File Structure

The generated OFD follows this structure:

```
output.ofd (ZIP)
├── OFD.xml                         # Entry point
└── Doc_0/
    ├── Document.xml                # Document root (pages, resources)
    ├── DocumentRes.xml             # Image resource declarations
    ├── PublicRes.xml               # Fonts, color spaces
    ├── Pages/
    │   ├── Page_0/Content.xml      # Page 0 content
    │   ├── Page_1/Content.xml      # Page 1 content
    │   └── ...
    └── Res/
        ├── Image_0.jpg             # Embedded image files
        ├── Image_1.jpg
        └── ...
```

## Roadmap

- [ ] **OFD Reader** - Parse and extract content from existing OFD files
- [ ] **Text rendering** - TextObject support with font embedding
- [ ] **Vector graphics** - PathObject for lines, curves, shapes
- [ ] **Digital signatures** - Electronic seal and signature support (GB/T 38540)
- [ ] **Template rendering** - Generate OFD from templates with dynamic data
- [ ] **PDF to OFD** - Convert PDF documents to OFD format
- [ ] **OFD to PDF** - Convert OFD documents to PDF format

## License

Licensed under Apache License 2.0. See [LICENSE](LICENSE) for details.
