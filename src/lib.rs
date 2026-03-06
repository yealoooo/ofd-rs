//! ofd-rs — Rust library for OFD (Open Fixed-layout Document, GB/T 33190-2016).
//!
//! Generate OFD documents from images. Structure follows
//! [ofdrw](https://github.com/ofdrw/ofdrw) conventions.
//!
//! # Quick Start
//!
//! ```no_run
//! use ofd_rs::{OfdWriter, ImageSource};
//!
//! let image_bytes = std::fs::read("photo.jpg").unwrap();
//! let ofd = OfdWriter::from_images(vec![
//!     ImageSource::auto_detect_default(image_bytes).unwrap(),
//! ]).build().unwrap();
//! std::fs::write("output.ofd", ofd).unwrap();
//! ```

pub mod types;
pub mod model;
mod writer;

pub use model::{DocInfo, ImageFormat, PageSize, PPM_DEFAULT};
pub use writer::{ImageSource, OfdError, OfdWriter};
