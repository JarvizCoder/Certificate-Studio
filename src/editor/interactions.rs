use eframe::egui::{CursorIcon, Pos2, Rect, Vec2};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResizeHandle {
    TopLeft,
    Top,
    TopRight,
    Right,
    BottomRight,
    Bottom,
    BottomLeft,
    Left,
}

impl ResizeHandle {
    pub fn cursor_icon(&self) -> CursorIcon {
        match self {
            Self::TopLeft | Self::BottomRight => CursorIcon::ResizeNwSe,
            Self::TopRight | Self::BottomLeft => CursorIcon::ResizeNeSw,
            Self::Top | Self::Bottom => CursorIcon::ResizeVertical,
            Self::Left | Self::Right => CursorIcon::ResizeHorizontal,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InteractionMode {
    #[default]
    None,
    DraggingElement(Uuid),
    ResizingElement(Uuid, ResizeHandle),
}

/// Transformation state between Document (in millimeters) and Screen coordinates (pixels).
#[derive(Debug, Clone)]
pub struct CanvasTransform {
    pub zoom: f32,
    pub pan: Vec2,
    /// Base pixels per millimeter at zoom = 1.0
    pub base_px_per_mm: f32,
}

impl Default for CanvasTransform {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            pan: Vec2::ZERO,
            base_px_per_mm: 3.5, // Standard preview density (~89 dpi display)
        }
    }
}

impl CanvasTransform {
    pub fn new(base_px_per_mm: f32) -> Self {
        Self {
            zoom: 1.0,
            pan: Vec2::ZERO,
            base_px_per_mm,
        }
    }

    #[inline]
    pub fn current_scale(&self) -> f32 {
        self.base_px_per_mm * self.zoom
    }

    /// Converts document coordinates (X mm, Y mm) relative to document top-left
    /// into screen pixel coordinates given the document's top-left screen position.
    #[inline]
    pub fn doc_mm_to_screen(&self, doc_origin_screen: Pos2, mm_pos: [f32; 2]) -> Pos2 {
        let scale = self.current_scale();
        Pos2::new(
            doc_origin_screen.x + mm_pos[0] * scale,
            doc_origin_screen.y + mm_pos[1] * scale,
        )
    }

    /// Converts screen pixel position into document coordinates (X mm, Y mm)
    /// relative to document top-left.
    #[inline]
    pub fn screen_to_doc_mm(&self, doc_origin_screen: Pos2, screen_pos: Pos2) -> [f32; 2] {
        let scale = self.current_scale();
        if scale <= 0.0 {
            return [0.0, 0.0];
        }
        [
            (screen_pos.x - doc_origin_screen.x) / scale,
            (screen_pos.y - doc_origin_screen.y) / scale,
        ]
    }

    /// Converts document dimensions in mm to screen size in pixels.
    #[inline]
    pub fn doc_size_to_screen(&self, mm_size: [f32; 2]) -> Vec2 {
        let scale = self.current_scale();
        Vec2::new(mm_size[0] * scale, mm_size[1] * scale)
    }

    /// Compute document rectangle on screen centered within the available viewport.
    pub fn compute_doc_rect(&self, viewport: Rect, doc_w_mm: f32, doc_h_mm: f32) -> Rect {
        let doc_screen_w = doc_w_mm * self.current_scale();
        let doc_screen_h = doc_h_mm * self.current_scale();

        let center = viewport.center() + self.pan;
        let min_x = center.x - (doc_screen_w / 2.0);
        let min_y = center.y - (doc_screen_h / 2.0);

        Rect::from_min_size(Pos2::new(min_x, min_y), Vec2::new(doc_screen_w, doc_screen_h))
    }

    /// Adjust zoom to neatly fit document in viewport with some padding.
    pub fn fit_to_viewport(&mut self, viewport: Rect, doc_w_mm: f32, doc_h_mm: f32) {
        if doc_w_mm <= 0.0 || doc_h_mm <= 0.0 {
            return;
        }
        let padding = 40.0;
        let avail_w = (viewport.width() - padding * 2.0).max(100.0);
        let avail_h = (viewport.height() - padding * 2.0).max(100.0);

        let scale_w = avail_w / doc_w_mm;
        let scale_h = avail_h / doc_h_mm;
        let target_scale = scale_w.min(scale_h);

        self.zoom = (target_scale / self.base_px_per_mm).clamp(0.1, 5.0);
        self.pan = Vec2::ZERO;
    }
}
