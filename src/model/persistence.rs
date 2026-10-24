use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use anyhow::{Context, Result};
use tracing::warn;

use super::project::CertificateProject;
use crate::fonts::manager::FontManager;

/// Result of loading a saved certificate project bundle.
#[derive(Debug)]
pub struct LoadedProjectBundle {
    pub project: CertificateProject,
    pub background_bytes: Option<Vec<u8>>,
    /// font_family_name -> (file_path, font_file_bytes)
    pub fonts: HashMap<String, (PathBuf, Vec<u8>)>,
    pub warnings: Vec<String>,
}

/// Save project manifest and package assets into a managed companion directory.
pub fn save_project_bundle(
    project: &CertificateProject,
    file_path: impl AsRef<Path>,
    font_manager: &FontManager,
) -> Result<()> {
    let file_path = file_path.as_ref();
    let parent_dir = file_path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Invalid file path without parent directory"))?;

    let file_stem = file_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("project");

    let assets_dir_name = format!("{}_assets", file_stem);
    let assets_dir = parent_dir.join(&assets_dir_name);
    let fonts_dir = assets_dir.join("fonts");

    fs::create_dir_all(&fonts_dir)
        .with_context(|| format!("Failed to create asset directories at {:?}", fonts_dir))?;

    let mut saved_project = project.clone();
    saved_project.font_assets.clear();

    // 1. Package Background Image
    if let Some(ref bg_path) = project.background_image_path {
        let ext = bg_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("png");
        let dest_bg_filename = format!("background.{}", ext);
        let dest_bg_path = assets_dir.join(&dest_bg_filename);

        // If not already saving onto itself, copy
        if bg_path.exists() && bg_path != &dest_bg_path {
            if let Err(err) = fs::copy(bg_path, &dest_bg_path) {
                warn!("Could not copy background image: {}", err);
            }
        }

        saved_project.background_image_path =
            Some(PathBuf::from(format!("{}/{}", assets_dir_name, dest_bg_filename)));
    }

    // 2. Package Custom Fonts
    for (font_name, entry) in &font_manager.loaded_fonts {
        let ext = entry
            .path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("ttf");
        let font_filename = format!("{}.{}", font_name, ext);
        let dest_font_path = fonts_dir.join(&font_filename);

        if !entry.bytes.is_empty() {
            if let Err(err) = fs::write(&dest_font_path, &entry.bytes) {
                warn!("Could not package font {}: {}", font_name, err);
            }
        } else if entry.path.exists() && entry.path != dest_font_path {
            if let Err(err) = fs::copy(&entry.path, &dest_font_path) {
                warn!("Could not copy font file {}: {}", font_name, err);
            }
        }

        let rel_font_path = PathBuf::from(format!("{}/fonts/{}", assets_dir_name, font_filename));
        saved_project
            .font_assets
            .insert(font_name.clone(), rel_font_path);
    }

    // 3. Write Project JSON Manifest
    let json_content = serde_json::to_string_pretty(&saved_project)
        .context("Failed to serialize project to JSON")?;

    fs::write(file_path, json_content)
        .with_context(|| format!("Failed to write project file to {:?}", file_path))?;

    Ok(())
}

/// Load project manifest and restore referenced background and font assets.
pub fn load_project_bundle(file_path: impl AsRef<Path>) -> Result<LoadedProjectBundle> {
    let file_path = file_path.as_ref();
    let parent_dir = file_path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Invalid project file path"))?;

    let json_content = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read project file at {:?}", file_path))?;

    let project: CertificateProject = serde_json::from_str(&json_content)
        .context("Invalid project JSON schema or corrupted file")?;

    let mut warnings = Vec::new();

    // 1. Load Background Image
    let mut bg_bytes = None;
    if let Some(ref rel_bg_path) = project.background_image_path {
        let resolved_path = parent_dir.join(rel_bg_path);
        if resolved_path.exists() {
            match fs::read(&resolved_path) {
                Ok(bytes) => bg_bytes = Some(bytes),
                Err(err) => warnings.push(format!("Could not read background image: {}", err)),
            }
        } else {
            warnings.push(format!(
                "Background asset missing: {:?}",
                resolved_path.file_name().unwrap_or_default()
            ));
        }
    }

    // 2. Load Custom Fonts
    let mut fonts = HashMap::new();
    for (font_name, rel_font_path) in &project.font_assets {
        let resolved_path = parent_dir.join(rel_font_path);
        if resolved_path.exists() {
            match fs::read(&resolved_path) {
                Ok(bytes) => {
                    fonts.insert(font_name.clone(), (resolved_path, bytes));
                }
                Err(err) => warnings.push(format!("Could not read font '{}': {}", font_name, err)),
            }
        } else {
            warnings.push(format!("Custom font asset missing: '{}'", font_name));
        }
    }

    Ok(LoadedProjectBundle {
        project,
        background_bytes: bg_bytes,
        fonts,
        warnings,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fonts::manager::CustomFontEntry;
    use crate::model::element::TextElement;

    #[test]
    fn test_save_and_reopen_bundle() {
        let temp_dir = std::env::temp_dir().join("cert_studio_test_bundle");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let project_file = temp_dir.join("test_cert.certproj");

        let mut project = CertificateProject::new("Annual Award");
        project.width_mm = 297.0;
        project.height_mm = 210.0;
        project.add_element(TextElement::new_default("Recipient", "Alice Smith", [50.0, 80.0]));

        let mut font_manager = FontManager::new();
        // Register mock font bytes
        font_manager.loaded_fonts.insert(
            "MockFont".to_string(),
            CustomFontEntry {
                name: "MockFont".to_string(),
                path: PathBuf::from("MockFont.ttf"),
                bytes: vec![1, 2, 3, 4, 5],
            },
        );

        // Save bundle
        save_project_bundle(&project, &project_file, &font_manager).expect("Save bundle should succeed");

        assert!(project_file.exists());
        let assets_dir = temp_dir.join("test_cert_assets");
        assert!(assets_dir.exists());
        assert!(assets_dir.join("fonts").join("MockFont.ttf").exists());

        // Reopen bundle
        let loaded = load_project_bundle(&project_file).expect("Load bundle should succeed");
        assert_eq!(loaded.project.name, "Annual Award");
        assert_eq!(loaded.project.elements.len(), 1);
        assert_eq!(loaded.project.elements[0].name, "Recipient");
        assert!(loaded.fonts.contains_key("MockFont"));
        assert_eq!(loaded.fonts.get("MockFont").unwrap().1, vec![1, 2, 3, 4, 5]);

        let _ = fs::remove_dir_all(&temp_dir);
    }
}

