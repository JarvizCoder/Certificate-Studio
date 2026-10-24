use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use anyhow::{Context, Result};
use eframe::egui::{Context as EguiContext, FontData, FontDefinitions, FontFamily};

#[derive(Clone)]
pub struct CustomFontEntry {
    pub name: String,
    pub path: PathBuf,
    pub bytes: Vec<u8>,
}

#[derive(Default)]
pub struct FontManager {
    pub loaded_fonts: HashMap<String, CustomFontEntry>,
}

impl FontManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Load and validate a font file from disk (TTF or OTF)
    pub fn load_font_from_path(&mut self, path: impl AsRef<Path>, ctx: &EguiContext) -> Result<String> {
        let path_ref = path.as_ref();
        let bytes = std::fs::read(path_ref)
            .with_context(|| format!("Failed to read font file at {:?}", path_ref))?;

        // Validate using fontdue parser
        let _font = fontdue::Font::from_bytes(bytes.as_slice(), fontdue::FontSettings::default())
            .map_err(|err| anyhow::anyhow!("Invalid font file {}: {}", path_ref.display(), err))?;

        let font_name = path_ref
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("CustomFont")
            .to_string();

        self.loaded_fonts.insert(
            font_name.clone(),
            CustomFontEntry {
                name: font_name.clone(),
                path: path_ref.to_path_buf(),
                bytes,
            },
        );

        // Register font with egui
        self.sync_with_egui(ctx);

        Ok(font_name)
    }

    /// Apply all registered custom fonts to egui font definitions
    pub fn sync_with_egui(&self, ctx: &EguiContext) {
        let mut defs = FontDefinitions::default();
        for (name, entry) in &self.loaded_fonts {
            defs.font_data.insert(
                name.clone(),
                Arc::new(FontData::from_owned(entry.bytes.clone())),
            );
            defs.families.insert(
                FontFamily::Name(name.as_str().into()),
                vec![name.clone(), "Proportional".to_string()],
            );
        }
        ctx.set_fonts(defs);
    }

    pub fn remove_font(&mut self, name: &str, ctx: &EguiContext) {
        self.loaded_fonts.remove(name);
        self.sync_with_egui(ctx);
    }

    pub fn available_fonts(&self) -> Vec<String> {
        let mut list = vec!["Default".to_string()];
        let mut custom: Vec<String> = self.loaded_fonts.keys().cloned().collect();
        custom.sort();
        list.extend(custom);
        list
    }
}
