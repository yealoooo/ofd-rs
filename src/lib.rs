//! ofd-rs — Rust library for OFD (Open Fixed-layout Document, GB/T 33190-2016).
//!
//! # Quick Start
//!
//! ```no_run
//! use ofd_rs::{OfdWriter, ImageSource};
//!
//! let jpeg_bytes = std::fs::read("photo.jpg").unwrap();
//! let ofd = OfdWriter::from_images(vec![
//!     ImageSource::jpeg(jpeg_bytes, 2480, 3508, 300.0),
//! ]).build().unwrap();
//! std::fs::write("output.ofd", ofd).unwrap();
//! ```

pub mod types;
pub mod model;
mod writer;

pub use model::{DocInfo, ImageData, ImageFormat, ImageObject, GraphicObject, PageSize};
pub use writer::{ImageSource, OfdError, OfdWriter};
