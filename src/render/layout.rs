//! Shared coordinate systems and layout constants.
//! All document layouts are canonically defined in millimeters.

pub const MM_PER_INCH: f32 = 25.4;
pub const POINTS_PER_INCH: f32 = 72.0;

/// Points (pt) per millimeter (mm)
pub const PT_PER_MM: f32 = POINTS_PER_INCH / MM_PER_INCH; // ~2.83464567

/// Default A4 Dimensions (Landscape) in millimeters
pub const A4_LANDSCAPE_WIDTH_MM: f32 = 297.0;
pub const A4_LANDSCAPE_HEIGHT_MM: f32 = 210.0;

/// Convert millimeters to typographical points (used by PDF engine)
#[inline]
pub fn mm_to_pt(mm: f32) -> f32 {
    mm * PT_PER_MM
}

/// Convert typographical points to millimeters
#[inline]
pub fn pt_to_mm(pt: f32) -> f32 {
    pt / PT_PER_MM
}

/// Convert millimeters to pixels at a given DPI
#[inline]
pub fn mm_to_px(mm: f32, dpi: f32) -> f32 {
    (mm / MM_PER_INCH) * dpi
}

/// Convert pixels to millimeters at a given DPI
#[inline]
pub fn px_to_mm(px: f32, dpi: f32) -> f32 {
    (px / dpi) * MM_PER_INCH
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_a4_dimensions() {
        assert_eq!(A4_LANDSCAPE_WIDTH_MM, 297.0);
        assert_eq!(A4_LANDSCAPE_HEIGHT_MM, 210.0);
    }

    #[test]
    fn test_unit_conversions() {
        let mm = 25.4;
        let pt = mm_to_pt(mm);
        assert!((pt - 72.0).abs() < 1e-4);
        assert!((pt_to_mm(pt) - 25.4).abs() < 1e-4);

        let px_300dpi = mm_to_px(25.4, 300.0);
        assert!((px_300dpi - 300.0).abs() < 1e-4);
    }
}

