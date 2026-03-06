//! Example: Convert a JPEG image to an OFD document.
//!
//! Usage: cargo run --example image_to_ofd <input.jpg> [output.ofd]

use ofd_rs::{ImageSource, OfdWriter};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <input.jpg> [output.ofd]", args[0]);
        std::process::exit(1);
    }

    let input_path = &args[1];
    let output_path = if args.len() > 2 {
        args[2].clone()
    } else {
        input_path.replace(".jpg", ".ofd").replace(".jpeg", ".ofd")
    };

    let jpeg_bytes = std::fs::read(input_path).expect("failed to read input file");

    // Default: A4 size at 150 DPI
    let ofd_bytes = OfdWriter::from_images(vec![
        ImageSource::jpeg_mm(jpeg_bytes, 210.0, 297.0),
    ])
    .build()
    .expect("failed to build OFD");

    std::fs::write(&output_path, ofd_bytes).expect("failed to write OFD file");
    println!("Created: {}", output_path);
}
