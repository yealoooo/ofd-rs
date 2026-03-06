//! Example: Convert an image (JPEG/PNG/BMP/TIFF) to an OFD document.
//!
//! Usage: cargo run --example image_to_ofd <input_image> [output.ofd]

use ofd_rs::{ImageSource, OfdWriter};
use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <input_image> [output.ofd]", args[0]);
        std::process::exit(1);
    }

    let input_path = &args[1];
    let output_path = if args.len() > 2 {
        args[2].clone()
    } else {
        let stem = Path::new(input_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output");
        let parent = Path::new(input_path)
            .parent()
            .unwrap_or(Path::new("."));
        parent.join(format!("{}.ofd", stem)).to_string_lossy().into_owned()
    };

    let image_bytes = std::fs::read(input_path).expect("failed to read input file");

    // Auto-detect format + dimensions, use ofdrw default ratio (5 px/mm ≈ 127 DPI)
    let source = ImageSource::auto_detect_default(image_bytes)
        .expect("unsupported image format or cannot read dimensions");

    println!(
        "Image: {}x{} mm (page size)",
        format!("{:.1}", source.page_size.width_mm),
        format!("{:.1}", source.page_size.height_mm),
    );

    let ofd_bytes = OfdWriter::from_images(vec![source])
        .build()
        .expect("failed to build OFD");

    std::fs::write(&output_path, &ofd_bytes).expect("failed to write OFD file");
    println!("Created: {} ({} bytes)", output_path, ofd_bytes.len());
}
