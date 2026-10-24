use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextAlignment {
    Left,
    Center,
    Right,
}

impl Default for TextAlignment {
    fn default() -> Self {
        Self::Center
    }
}

/// Represents an individual text element on the certificate canvas.
/// Coordinates and dimensions are strictly in millimeters.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextElement {
    pub id: Uuid,
    pub name: String,
    pub text: String,
    /// [X, Y] in millimeters from top-left of the document.
    pub pos_mm: [f32; 2],
    /// [Width, Height] in millimeters.
    pub size_mm: [f32; 2],
    pub font_family: String,
    pub font_size_pt: f32,
    pub color_rgba: [u8; 4],
    pub alignment: TextAlignment,
    /// CSV placeholder key (e.g., "name", "certificate_id")
    pub field_key: Option<String>,
    pub is_locked: bool,
    pub is_visible: bool,
}

impl TextElement {
    pub fn new_default(name: impl Into<String>, text: impl Into<String>, pos_mm: [f32; 2]) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            text: text.into(),
            pos_mm,
            size_mm: [120.0, 20.0],
            font_family: "Default".to_string(),
            font_size_pt: 24.0,
            color_rgba: [30, 30, 30, 255],
            alignment: TextAlignment::Center,
            field_key: None,
            is_locked: false,
            is_visible: true,
        }
    }
}

