use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use super::element::TextElement;
use crate::render::layout::{A4_LANDSCAPE_HEIGHT_MM, A4_LANDSCAPE_WIDTH_MM};

/// Master certificate template model.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CertificateProject {
    pub id: Uuid,
    pub name: String,
    pub width_mm: f32,
    pub height_mm: f32,
    /// Path to background image (relative to project or absolute for workspace)
    pub background_image_path: Option<PathBuf>,
    pub elements: Vec<TextElement>,
}

impl Default for CertificateProject {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            name: "Untitled Certificate".to_string(),
            width_mm: A4_LANDSCAPE_WIDTH_MM,
            height_mm: A4_LANDSCAPE_HEIGHT_MM,
            background_image_path: None,
            elements: Vec::new(),
        }
    }
}

impl CertificateProject {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    pub fn aspect_ratio(&self) -> f32 {
        if self.height_mm <= 0.0 {
            1.0
        } else {
            self.width_mm / self.height_mm
        }
    }

    pub fn add_element(&mut self, element: TextElement) {
        self.elements.push(element);
    }

    pub fn remove_element(&mut self, id: &Uuid) {
        self.elements.retain(|el| &el.id != id);
    }

    pub fn get_element_mut(&mut self, id: &Uuid) -> Option<&mut TextElement> {
        self.elements.iter_mut().find(|el| &el.id == id)
    }
}

