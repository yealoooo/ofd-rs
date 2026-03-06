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
        write!(f, "{} {} {} {}", format_mm(self.x), format_mm(self.y), format_mm(self.w), format_mm(self.h))
    }
}

/// Format a mm value: drop trailing zeros, integers show as "210" not "210.0000".
/// Matches ofdrw behavior: `0 0 210 297` for round numbers, `227.1889` for fractional.
fn format_mm(v: f64) -> String {
    if v.fract() == 0.0 {
        format!("{}", v as i64)
    } else {
        // Up to 4 decimal places, trim trailing zeros
        let s = format!("{:.4}", v);
        s.trim_end_matches('0').trim_end_matches('.').to_string()
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
        write!(f, "{} {}", format_mm(self.x), format_mm(self.y))
    }
}

/// Format mm value for use outside types.rs (e.g., CTM attributes).
pub fn format_mm_value(v: f64) -> String {
    format_mm(v)
}

/// ST_ID: Unsigned integer identifier, unique within a document.
pub type StId = u32;

/// ST_RefID: Reference to an existing ST_ID.
pub type StRefId = u32;

/// OFD namespace URI (GB/T 33190-2016 Section 7.1).
pub const OFD_NAMESPACE: &str = "http://www.ofdspec.org/2016";
