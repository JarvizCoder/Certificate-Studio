use std::collections::HashMap;
use std::path::PathBuf;
use eframe::egui::{self, Color32, Panel, CentralPanel, RichText, TextureHandle, Ui, Window};
use uuid::Uuid;

use crate::data::csv_import::CsvDataset;
use crate::data::field_mapping::extract_placeholders;
use crate::editor::canvas::CertificateCanvas;
use crate::editor::history::ProjectHistory;
use crate::editor::interactions::{CanvasTransform, InteractionMode};
use crate::editor::properties::PropertiesPanel;
use crate::fonts::manager::FontManager;
use crate::model::element::TextElement;
use crate::model::persistence::{load_project_bundle, save_project_bundle};
use crate::model::project::CertificateProject;

pub struct CertificateStudioApp {
    pub project: CertificateProject,
    pub current_project_path: Option<PathBuf>,
    pub is_dirty: bool,
    pub history: ProjectHistory,
    pub transform: CanvasTransform,
    pub font_manager: FontManager,
    pub selected_element_id: Option<Uuid>,
    pub interaction_mode: InteractionMode,
    pub status_hover_coord_mm: Option<[f32; 2]>,
    pub bg_texture: Option<TextureHandle>,
    pub bg_dimensions: Option<(u32, u32)>,
    pub status_message: String,
    pub fit_requested: bool,
    // Milestone 4: CSV Data & Personalization
    pub csv_dataset: Option<CsvDataset>,
    pub preview_record_idx: Option<usize>,
    pub show_csv_table_window: bool,
    pub validation_summary: Option<String>,
}

impl Default for CertificateStudioApp {
    fn default() -> Self {
        Self {
            project: CertificateProject::default(),
            current_project_path: None,
            is_dirty: false,
            history: ProjectHistory::default(),
            transform: CanvasTransform::default(),
            font_manager: FontManager::default(),
            selected_element_id: None,
            interaction_mode: InteractionMode::default(),
            status_hover_coord_mm: None,
            bg_texture: None,
            bg_dimensions: None,
            status_message: "Ready. Import a background image or add text to begin.".to_string(),
            fit_requested: false,
            csv_dataset: None,
            preview_record_idx: None,
            show_csv_table_window: false,
            validation_summary: None,
        }
    }
}

impl CertificateStudioApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self::default()
    }

    pub fn new_project(&mut self) {
        self.history.clear();
        self.project = CertificateProject::default();
        self.current_project_path = None;
        self.bg_texture = None;
        self.bg_dimensions = None;
        self.selected_element_id = None;
        self.is_dirty = false;
        self.validation_summary = None;
        self.status_message = "Created new project.".to_string();
    }

    pub fn save_project(&mut self) {
        if let Some(ref path) = self.current_project_path.clone() {
            match save_project_bundle(&self.project, path, &self.font_manager) {
                Ok(()) => {
                    self.is_dirty = false;
                    let fname = path.file_name().and_then(|f| f.to_str()).unwrap_or("project");
                    self.status_message = format!("Saved project to '{}'", fname);
                }
                Err(err) => {
                    self.status_message = format!("Failed to save project: {}", err);
                }
            }
        } else {
            self.save_project_as();
        }
    }

    pub fn save_project_as(&mut self) {
        let suggested_name = format!("{}.certproj", self.project.name.to_lowercase().replace(' ', "_"));
        let dialog = rfd::FileDialog::new()
            .set_title("Save Certificate Project As")
            .set_file_name(&suggested_name)
            .add_filter("Certificate Project (*.certproj)", &["certproj", "json"]);

        if let Some(path) = dialog.save_file() {
            match save_project_bundle(&self.project, &path, &self.font_manager) {
                Ok(()) => {
                    self.current_project_path = Some(path.clone());
                    self.is_dirty = false;
                    let fname = path.file_name().and_then(|f| f.to_str()).unwrap_or("project");
                    self.status_message = format!("Saved project to '{}'", fname);
                }
                Err(err) => {
                    self.status_message = format!("Failed to save project: {}", err);
                }
            }
        }
    }

    pub fn open_project(&mut self, ctx: &egui::Context) {
        let dialog = rfd::FileDialog::new()
            .set_title("Open Certificate Project")
            .add_filter("Certificate Project (*.certproj, *.json)", &["certproj", "json"]);

        if let Some(path) = dialog.pick_file() {
            match load_project_bundle(&path) {
                Ok(loaded) => {
                    self.project = loaded.project;
                    self.current_project_path = Some(path.clone());
                    self.selected_element_id = None;
                    self.history.clear();
                    self.is_dirty = false;

                    // Restore background image if present
                    if let Some(bytes) = loaded.background_bytes {
                        if let Ok(dyn_img) = image::load_from_memory(&bytes) {
                            let (w, h) = (dyn_img.width(), dyn_img.height());
                            let rgba = dyn_img.to_rgba8();
                            let color_image = egui::ColorImage::from_rgba_unmultiplied(
                                [w as usize, h as usize],
                                rgba.as_flat_samples().as_slice(),
                            );
                            let texture = ctx.load_texture(
                                "certificate_bg",
                                color_image,
                                egui::TextureOptions::LINEAR,
                            );
                            self.bg_texture = Some(texture);
                            self.bg_dimensions = Some((w, h));
                        }
                    } else {
                        self.bg_texture = None;
                        self.bg_dimensions = None;
                    }

                    // Restore custom fonts
                    for (font_name, (font_path, font_bytes)) in loaded.fonts {
                        self.font_manager.loaded_fonts.insert(
                            font_name.clone(),
                            crate::fonts::manager::CustomFontEntry {
                                name: font_name,
                                path: font_path,
                                bytes: font_bytes,
                            },
                        );
                    }
                    self.font_manager.sync_with_egui(ctx);

                    let fname = path.file_name().and_then(|f| f.to_str()).unwrap_or("project");
                    if !loaded.warnings.is_empty() {
                        self.status_message = format!(
                            "Opened '{}' with {} warnings: {}",
                            fname,
                            loaded.warnings.len(),
                            loaded.warnings.join("; ")
                        );
                    } else {
                        self.status_message = format!("Opened project '{}'", fname);
                    }
                }
                Err(err) => {
                    self.status_message = format!("Failed to open project: {}", err);
                }
            }
        }
    }

    /// Open native file dialog to import a background certificate image (PNG / JPG)
    pub fn import_background(&mut self, ctx: &egui::Context) {
        let dialog = rfd::FileDialog::new()
            .set_title("Select Certificate Background Image")
            .add_filter("Image Files", &["png", "jpg", "jpeg", "webp", "bmp"]);

        if let Some(path) = dialog.pick_file() {
            self.history.push_snapshot(&self.project);
            self.is_dirty = true;
            self.load_background_from_path(ctx, path);
        }
    }

    pub fn load_background_from_path(&mut self, ctx: &egui::Context, path: PathBuf) {
        match std::fs::read(&path) {
            Ok(bytes) => match image::load_from_memory(&bytes) {
                Ok(dyn_img) => {
                    let (w, h) = (dyn_img.width(), dyn_img.height());
                    let rgba = dyn_img.to_rgba8();
                    let color_image = egui::ColorImage::from_rgba_unmultiplied(
                        [w as usize, h as usize],
                        rgba.as_flat_samples().as_slice(),
                    );

                    let texture = ctx.load_texture(
                        "certificate_bg",
                        color_image,
                        egui::TextureOptions::LINEAR,
                    );

                    self.bg_texture = Some(texture);
                    self.bg_dimensions = Some((w, h));
                    self.project.background_image_path = Some(path.clone());

                    let file_name = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("image");

                    self.status_message = format!(
                        "Loaded background '{}' ({} × {} px)",
                        file_name, w, h
                    );
                }
                Err(err) => {
                    self.status_message = format!("Failed to decode image: {}", err);
                }
            },
            Err(err) => {
                self.status_message = format!("Failed to read file: {}", err);
            }
        }
    }

    /// Open native file dialog to import custom TTF or OTF fonts
    pub fn import_font(&mut self, ctx: &egui::Context) {
        let dialog = rfd::FileDialog::new()
            .set_title("Select Font File")
            .add_filter("TrueType & OpenType Fonts", &["ttf", "otf"]);

        if let Some(path) = dialog.pick_file() {
            match self.font_manager.load_font_from_path(&path, ctx) {
                Ok(font_name) => {
                    self.is_dirty = true;
                    self.status_message = format!("Successfully imported font '{}'", font_name);
                }
                Err(err) => {
                    self.status_message = format!("Font import error: {}", err);
                }
            }
        }
    }

    /// Open native file dialog to import a CSV dataset
    pub fn import_csv(&mut self) {
        let dialog = rfd::FileDialog::new()
            .set_title("Import CSV Dataset")
            .add_filter("CSV Files (*.csv)", &["csv"]);

        if let Some(path) = dialog.pick_file() {
            match CsvDataset::parse_from_path(&path) {
                Ok(dataset) => {
                    let total = dataset.total_records();
                    let cols = dataset.headers.len();
                    let fname = dataset.filename.clone();
                    self.preview_record_idx = if total > 0 { Some(0) } else { None };
                    self.csv_dataset = Some(dataset);
                    self.validation_summary = None;
                    self.status_message = format!("Imported CSV '{}': {} records, {} columns", fname, total, cols);
                }
                Err(err) => {
                    self.status_message = format!("CSV Import Error: {}", err);
                }
            }
        }
    }

    pub fn validate_current_dataset(&mut self) {
        if let Some(ref dataset) = self.csv_dataset {
            // Collect all required placeholder keys and field bindings from project
            let mut required_keys = Vec::new();
            for elem in &self.project.elements {
                if let Some(ref k) = elem.field_key {
                    if !required_keys.contains(k) {
                        required_keys.push(k.clone());
                    }
                }
                for placeholder in extract_placeholders(&elem.text) {
                    if !required_keys.contains(&placeholder) {
                        required_keys.push(placeholder);
                    }
                }
            }

            if required_keys.is_empty() {
                self.validation_summary = Some("No dynamic fields bound yet. Bind text elements to CSV columns to test records.".to_string());
                return;
            }

            let failures = dataset.validate_required_keys(&required_keys);
            if failures.is_empty() {
                self.validation_summary = Some(format!(
                    "✅ All {} records valid! All bound fields ({}) are present.",
                    dataset.total_records(),
                    required_keys.join(", ")
                ));
            } else {
                let first_few = failures
                    .iter()
                    .take(3)
                    .map(|(row, missing)| format!("Row {}: missing [{}]", row, missing.join(", ")))
                    .collect::<Vec<_>>()
                    .join("; ");
                self.validation_summary = Some(format!(
                    "⚠️ {} of {} records have missing values: {}{}",
                    failures.len(),
                    dataset.total_records(),
                    first_few,
                    if failures.len() > 3 { "..." } else { "" }
                ));
            }
        }
    }

    /// Retrieve the currently selected CSV record for canvas live preview
    pub fn current_preview_record(&self) -> Option<HashMap<String, String>> {
        let idx = self.preview_record_idx?;
        let dataset = self.csv_dataset.as_ref()?;
        dataset.row_as_map(idx)
    }

    pub fn add_sample_text(&mut self) {
        self.history.push_snapshot(&self.project);
        self.is_dirty = true;
        let count = self.project.elements.len() + 1;
        let x = (self.project.width_mm - 120.0) / 2.0;
        let y = 50.0 + (count as f32 * 15.0).min(self.project.height_mm - 60.0);
        let new_elem = TextElement::new_default(
            format!("Text Element {}", count),
            "Certificate of Excellence",
            [x, y],
        );
        let id = new_elem.id;
        self.project.add_element(new_elem);
        self.selected_element_id = Some(id);
        self.status_message = format!("Added text element {}", count);
    }
}

impl eframe::App for CertificateStudioApp {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        // Keyboard shortcuts
        if ui.input(|i| i.modifiers.command && i.key_pressed(egui::Key::Z) && !i.modifiers.shift) {
            if self.history.undo(&mut self.project) {
                self.is_dirty = true;
                self.status_message = "Undo".to_string();
            }
        }
        if ui.input(|i| {
            (i.modifiers.command && i.key_pressed(egui::Key::Y))
                || (i.modifiers.command && i.modifiers.shift && i.key_pressed(egui::Key::Z))
        }) {
            if self.history.redo(&mut self.project) {
                self.is_dirty = true;
                self.status_message = "Redo".to_string();
            }
        }
        if ui.input(|i| i.modifiers.command && !i.modifiers.shift && i.key_pressed(egui::Key::S)) {
            self.save_project();
        }
        if ui.input(|i| i.modifiers.command && i.modifiers.shift && i.key_pressed(egui::Key::S)) {
            self.save_project_as();
        }
        if ui.input(|i| i.modifiers.command && i.key_pressed(egui::Key::O)) {
            self.open_project(&ctx);
        }
        if ui.input(|i| i.modifiers.command && i.key_pressed(egui::Key::N)) {
            self.new_project();
        }

        // --- 1. Top Toolbar ---
        Panel::top("top_toolbar").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Certificate Studio");

                let title_text = format!(
                    " - {}{}",
                    self.project.name,
                    if self.is_dirty { " *" } else { "" }
                );
                ui.label(RichText::new(title_text).strong().color(Color32::from_rgb(180, 210, 240)));

                ui.separator();

                if ui.button("📄 New").on_hover_text("Start a fresh certificate project (Ctrl+N)").clicked() {
                    self.new_project();
                }

                if ui.button("📂 Open").on_hover_text("Open saved project bundle (Ctrl+O)").clicked() {
                    self.open_project(&ctx);
                }

                if ui.button("💾 Save").on_hover_text("Save project (Ctrl+S)").clicked() {
                    self.save_project();
                }

                if ui.button("💾 Save As...").on_hover_text("Save project as a new bundle (Ctrl+Shift+S)").clicked() {
                    self.save_project_as();
                }

                ui.separator();

                // Undo / Redo controls
                ui.add_enabled_ui(self.history.can_undo(), |ui| {
                    if ui.button("⮌ Undo").on_hover_text("Undo change (Ctrl+Z)").clicked() {
                        if self.history.undo(&mut self.project) {
                            self.is_dirty = true;
                            self.status_message = "Undo".to_string();
                        }
                    }
                });

                ui.add_enabled_ui(self.history.can_redo(), |ui| {
                    if ui.button("⮎ Redo").on_hover_text("Redo change (Ctrl+Y)").clicked() {
                        if self.history.redo(&mut self.project) {
                            self.is_dirty = true;
                            self.status_message = "Redo".to_string();
                        }
                    }
                });

                ui.separator();

                // Live recipient preview switcher in toolbar
                if let Some(ref ds) = self.csv_dataset {
                    let total = ds.total_records();
                    if total > 0 {
                        ui.label(RichText::new("Preview Recipient:").strong());
                        let current_idx = self.preview_record_idx.unwrap_or(0);

                        if ui.button("◀").on_hover_text("Previous recipient record").clicked() {
                            let next_idx = if current_idx == 0 { total - 1 } else { current_idx - 1 };
                            self.preview_record_idx = Some(next_idx);
                        }

                        let preview_label = format!("{}/{}", current_idx + 1, total);
                        ui.label(RichText::new(preview_label).color(Color32::from_rgb(120, 200, 255)));

                        if ui.button("▶").on_hover_text("Next recipient record").clicked() {
                            let next_idx = (current_idx + 1) % total;
                            self.preview_record_idx = Some(next_idx);
                        }

                        let mut is_previewing = self.preview_record_idx.is_some();
                        if ui.checkbox(&mut is_previewing, "Live Canvas").clicked() {
                            self.preview_record_idx = if is_previewing { Some(0) } else { None };
                        }

                        ui.separator();
                    }
                }

                // Zoom controls
                if ui.button("➖ Zoom -").clicked() {
                    self.transform.zoom = (self.transform.zoom * 0.85).max(0.1);
                }
                ui.label(format!("{:.0}%", self.transform.zoom * 100.0));
                if ui.button("➕ Zoom +").clicked() {
                    self.transform.zoom = (self.transform.zoom * 1.15).min(5.0);
                }
                if ui.button("100%").clicked() {
                    self.transform.zoom = 1.0;
                    self.transform.pan = egui::Vec2::ZERO;
                }
                if ui.button("🔍 Fit").on_hover_text("Fit certificate to current view").clicked() {
                    self.fit_requested = true;
                }

                ui.separator();
                ui.label(
                    RichText::new(format!(
                        "Size: {:.0} × {:.0} mm",
                        self.project.width_mm, self.project.height_mm
                    ))
                    .color(Color32::from_gray(160)),
                );
            });
        });

        // --- 2. Bottom Status Bar ---
        Panel::bottom("bottom_status_bar").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(&self.status_message).size(12.0));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if let Some([x, y]) = self.status_hover_coord_mm {
                        ui.label(
                            RichText::new(format!("X: {:.1} mm | Y: {:.1} mm", x, y))
                                .size(12.0)
                                .color(Color32::from_rgb(100, 180, 240)),
                        );
                    }
                    if let Some((w, h)) = self.bg_dimensions {
                        ui.label(
                            RichText::new(format!("BG: {}×{} px", w, h))
                                .size(12.0)
                                .color(Color32::from_gray(140)),
                        );
                    }
                    if let Some(ref path) = self.current_project_path {
                        let fname = path.file_name().and_then(|f| f.to_str()).unwrap_or("project");
                        ui.label(
                            RichText::new(format!("📁 {}", fname))
                                .size(12.0)
                                .color(Color32::from_gray(150)),
                        );
                    }
                    if let Some(ref ds) = self.csv_dataset {
                        ui.label(
                            RichText::new(format!("📊 CSV: {} records", ds.total_records()))
                                .size(12.0)
                                .color(Color32::from_rgb(120, 200, 120)),
                        );
                    }
                });
            });
        });

        // --- 3. Left Sidebar ---
        Panel::left("left_sidebar")
            .resizable(true)
            .default_size(240.0)
            .size_range(200.0..=380.0)
            .show(ui, |ui| {
                ui.heading("Design Assets");
                ui.separator();

                ui.label(RichText::new("Background Image").strong());
                ui.add_space(2.0);
                if ui
                    .button("🖼 Import Background")
                    .on_hover_text("Load a certificate background image (PNG, JPG)")
                    .clicked()
                {
                    self.import_background(&ctx);
                }

                if let Some(ref path) = self.project.background_image_path {
                    let fname = path.file_name().and_then(|f| f.to_str()).unwrap_or("image");
                    ui.add_space(2.0);
                    ui.label(RichText::new(format!("File: {}", fname)).size(11.0).color(Color32::from_rgb(120, 200, 120)));
                    if let Some((w, h)) = self.bg_dimensions {
                        ui.label(RichText::new(format!("Dimensions: {} × {} px", w, h)).size(11.0).color(Color32::from_gray(140)));
                    }
                    if ui.button("Remove Background").clicked() {
                        self.history.push_snapshot(&self.project);
                        self.is_dirty = true;
                        self.bg_texture = None;
                        self.bg_dimensions = None;
                        self.project.background_image_path = None;
                        self.status_message = "Background image removed.".to_string();
                    }
                } else {
                    ui.label(RichText::new("No background image loaded").size(11.0).italics().color(Color32::from_gray(140)));
                }

                ui.add_space(10.0);
                ui.separator();

                // Custom Fonts section
                ui.label(RichText::new("Custom Fonts").strong());
                ui.add_space(2.0);
                if ui.button("🔤 Import Font (.ttf / .otf)").clicked() {
                    self.import_font(&ctx);
                }

                let custom_font_names: Vec<String> = self.font_manager.loaded_fonts.keys().cloned().collect();
                if custom_font_names.is_empty() {
                    ui.label(RichText::new("No custom fonts loaded").size(11.0).italics().color(Color32::from_gray(140)));
                } else {
                    let mut remove_font_name = None;
                    for font_name in &custom_font_names {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(font_name).size(12.0));
                            if ui.small_button("✖").on_hover_text("Remove font").clicked() {
                                remove_font_name = Some(font_name.clone());
                            }
                        });
                    }
                    if let Some(name) = remove_font_name {
                        self.is_dirty = true;
                        self.font_manager.remove_font(&name, &ctx);
                        self.status_message = format!("Removed font '{}'", name);
                    }
                }

                ui.add_space(10.0);
                ui.separator();

                // Milestone 4: Participant Data (CSV) section
                ui.label(RichText::new("Participant Data (CSV)").strong());
                ui.add_space(2.0);

                if let Some(ref ds) = self.csv_dataset {
                    ui.label(RichText::new(format!("📄 {}", ds.filename)).size(11.0).color(Color32::from_rgb(120, 200, 120)));
                    ui.label(RichText::new(format!("{} records, {} columns", ds.total_records(), ds.headers.len())).size(11.0).color(Color32::from_gray(140)));

                    ui.horizontal(|ui| {
                        if ui.button("📋 View Table").on_hover_text("Open interactive CSV data table").clicked() {
                            self.show_csv_table_window = true;
                        }
                        if ui.button("⚠️ Validate").on_hover_text("Check for empty or missing values").clicked() {
                            self.validate_current_dataset();
                        }
                    });

                    if let Some(ref summary) = self.validation_summary {
                        ui.add_space(2.0);
                        ui.label(RichText::new(summary).size(11.0));
                    }

                    if ui.button("Clear CSV").clicked() {
                        self.csv_dataset = None;
                        self.preview_record_idx = None;
                        self.validation_summary = None;
                        self.status_message = "Cleared CSV dataset.".to_string();
                    }
                } else {
                    if ui.button("📊 Import CSV Dataset").on_hover_text("Load participant data (e.g. name, event, id)").clicked() {
                        self.import_csv();
                    }
                    ui.label(RichText::new("No CSV loaded").size(11.0).italics().color(Color32::from_gray(140)));
                }

                ui.add_space(12.0);
                ui.separator();
                ui.heading("Elements");

                ui.add_space(4.0);
                if ui.button("➕ Add Text").clicked() {
                    self.add_sample_text();
                }

                ui.add_space(8.0);
                egui::ScrollArea::vertical().show(ui, |ui| {
                    let mut delete_id = None;
                    for elem in &mut self.project.elements {
                        let is_sel = self.selected_element_id == Some(elem.id);
                        ui.horizontal(|ui| {
                            let eye_label = if elem.is_visible { "👁" } else { "🚫" };
                            if ui.small_button(eye_label).clicked() {
                                elem.is_visible = !elem.is_visible;
                            }
                            if ui.selectable_label(is_sel, &elem.name).clicked() {
                                self.selected_element_id = Some(elem.id);
                            }
                            if ui.small_button("🗑").on_hover_text("Delete element").clicked() {
                                delete_id = Some(elem.id);
                            }
                        });
                    }
                    if let Some(del_id) = delete_id {
                        self.history.push_snapshot(&self.project);
                        self.is_dirty = true;
                        self.project.remove_element(&del_id);
                        if self.selected_element_id == Some(del_id) {
                            self.selected_element_id = None;
                        }
                    }
                });
            });

        // --- 4. Right Sidebar: Properties ---
        Panel::right("right_sidebar")
            .resizable(true)
            .default_size(270.0)
            .size_range(220.0..=420.0)
            .show(ui, |ui| {
                let available_fonts = self.font_manager.available_fonts();
                let available_csv_cols = self
                    .csv_dataset
                    .as_ref()
                    .map(|d| d.headers.clone())
                    .unwrap_or_default();

                PropertiesPanel::show(
                    ui,
                    &mut self.project,
                    &mut self.selected_element_id,
                    &available_fonts,
                    &available_csv_cols,
                );
            });

        // --- 5. Center Panel: Canvas ---
        CentralPanel::default().show(ui, |ui| {
            if self.fit_requested {
                self.transform.fit_to_viewport(
                    ui.available_rect_before_wrap(),
                    self.project.width_mm,
                    self.project.height_mm,
                );
                self.fit_requested = false;
            }

            let active_record = self.current_preview_record();

            CertificateCanvas::show(
                ui,
                &mut self.project,
                self.bg_texture.as_ref(),
                &mut self.transform,
                &mut self.selected_element_id,
                &mut self.interaction_mode,
                &mut self.history,
                active_record.as_ref(),
                &mut self.status_hover_coord_mm,
            );
        });

        // --- 6. CSV Data Table Modal Window ---
        if self.show_csv_table_window {
            let mut is_open = self.show_csv_table_window;
            let ds_opt = self.csv_dataset.clone();

            Window::new("CSV Dataset Preview")
                .open(&mut is_open)
                .default_size([700.0, 400.0])
                .show(&ctx, |ui| {
                    if let Some(ref ds) = ds_opt {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(format!("File: {}", ds.filename)).strong());
                            ui.label(format!("({} records, {} columns)", ds.total_records(), ds.headers.len()));
                        });
                        ui.separator();

                        egui::ScrollArea::both().show(ui, |ui| {
                            egui::Grid::new("csv_preview_grid")
                                .striped(true)
                                .spacing([12.0, 6.0])
                                .show(ui, |ui| {
                                    ui.label(RichText::new("#").strong());
                                    for h in &ds.headers {
                                        ui.label(RichText::new(h).strong().color(Color32::from_rgb(120, 180, 240)));
                                    }
                                    ui.end_row();

                                    for (i, row) in ds.rows.iter().enumerate() {
                                        let is_current = self.preview_record_idx == Some(i);
                                        let num_label = if is_current {
                                            RichText::new(format!("▶ {}", i + 1)).strong().color(Color32::from_rgb(100, 220, 100))
                                        } else {
                                            RichText::new(format!("{}", i + 1)).color(Color32::from_gray(160))
                                        };

                                        if ui.selectable_label(is_current, num_label).on_hover_text("Click to preview on canvas").clicked() {
                                            self.preview_record_idx = Some(i);
                                        }

                                        for cell in row {
                                            ui.label(cell);
                                        }
                                        ui.end_row();
                                    }
                                });
                        });
                    }
                });
            self.show_csv_table_window = is_open;
        }
    }
}
