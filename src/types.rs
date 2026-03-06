//! Basic data types defined in GB/T 33190-2016 Section 7.3.

use std::fmt;

/// ST_Box: Rectangle area — "x y w h" in millimeters.
#[derive(Debug, Clone, Copy)]
pub struct StBox {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

impl StBox {
    pub fn new(x: f64, y: f64, w: f64, h: f64) -> Self {
        Self { x, y, w, h }
    }

    /// Full-page box starting at origin.
    pub fn page(w: f64, h: f64) -> Self {
        Self { x: 0.0, y: 0.0, w, h }
    }
}

impl fmt::Display for StBox {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.4} {:.4} {:.4} {:.4}", self.x, self.y, self.w, self.h)
    }
}

/// ST_Pos: Point coordinate — "x y".
#[derive(Debug, Clone, Copy)]
pub struct StPos {
    pub x: f64,
    pub y: f64,
}

impl fmt::Display for StPos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.4} {:.4}", self.x, self.y)
    }
}

/// ST_ID: Unsigned integer identifier, unique within a document.
pub type StId = u32;

/// ST_RefID: Reference to an existing ST_ID.
pub type StRefId = u32;

/// OFD namespace URI (GB/T 33190-2016 Section 7.1).
pub const OFD_NAMESPACE: &str = "http://www.ofdspec.org/2016";
