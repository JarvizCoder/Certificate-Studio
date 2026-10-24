use eframe::egui::{self, Color32, Ui};
use uuid::Uuid;
use crate::model::element::{TextAlignment, TextElement};
use crate::model::project::CertificateProject;
use crate::render::layout::{A4_LANDSCAPE_HEIGHT_MM, A4_LANDSCAPE_WIDTH_MM};

pub struct PropertiesPanel;

impl PropertiesPanel {
    pub fn show(
        ui: &mut Ui,
        project: &mut CertificateProject,
        selected_element_id: &mut Option<Uuid>,
        available_fonts: &[String],
        available_csv_columns: &[String],
    ) {
        ui.heading("Properties");
        ui.separator();

        // Project / Document Page Settings
        ui.collapsing("Page Setup", |ui| {
            ui.horizontal(|ui| {
                ui.label("Name:");
                ui.text_edit_singleline(&mut project.name);
            });

            ui.add_space(4.0);
            ui.label("Presets:");
            ui.horizontal(|ui| {
                if ui.button("A4 Landscape").clicked() {
                    project.width_mm = A4_LANDSCAPE_WIDTH_MM;
                    project.height_mm = A4_LANDSCAPE_HEIGHT_MM;
                }
                if ui.button("A4 Portrait").clicked() {
                    project.width_mm = A4_LANDSCAPE_HEIGHT_MM;
                    project.height_mm = A4_LANDSCAPE_WIDTH_MM;
                }
            });

            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label("Width (mm):");
                ui.add(egui::DragValue::new(&mut project.width_mm).speed(1.0).range(50.0..=1000.0));
            });
            ui.horizontal(|ui| {
                ui.label("Height (mm):");
                ui.add(egui::DragValue::new(&mut project.height_mm).speed(1.0).range(50.0..=1000.0));
            });
            ui.label(format!("Aspect ratio: {:.2}:1", project.aspect_ratio()));
        });

        ui.add_space(8.0);
        ui.separator();

        // Element properties
        if let Some(id) = *selected_element_id {
            if let Some(elem) = project.get_element_mut(&id) {
                Self::show_element_properties(ui, elem, available_fonts, available_csv_columns);

                ui.add_space(10.0);
                if ui.button("🗑 Delete Element").clicked() {
                    project.remove_element(&id);
                    *selected_element_id = None;
                }
            } else {
                *selected_element_id = None;
                Self::show_empty_selection_placeholder(ui);
            }
        } else {
            Self::show_empty_selection_placeholder(ui);
        }
    }

    fn show_empty_selection_placeholder(ui: &mut Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);
            ui.label(
                egui::RichText::new("No element selected")
                    .color(Color32::from_gray(140))
                    .italics(),
            );
            ui.label(
                egui::RichText::new("Select an element on the canvas or from the left sidebar to edit typography and layout.")
                    .size(11.0)
                    .color(Color32::from_gray(120)),
            );
        });
    }

    fn show_element_properties(
        ui: &mut Ui,
        elem: &mut TextElement,
        available_fonts: &[String],
        available_csv_columns: &[String],
    ) {
        ui.heading("Text Element");
        ui.add_space(4.0);

        ui.horizontal(|ui| {
            ui.label("Label:");
            ui.text_edit_singleline(&mut elem.name);
        });

        ui.label("Content (supports {{key}}):");
        ui.text_edit_multiline(&mut elem.text);

        ui.add_space(4.0);
        ui.collapsing("Position & Dimensions (mm)", |ui| {
            ui.horizontal(|ui| {
                ui.label("X (mm):");
                ui.add(egui::DragValue::new(&mut elem.pos_mm[0]).speed(0.5));
                ui.label("Y (mm):");
                ui.add(egui::DragValue::new(&mut elem.pos_mm[1]).speed(0.5));
            });
            ui.horizontal(|ui| {
                ui.label("W (mm):");
                ui.add(egui::DragValue::new(&mut elem.size_mm[0]).speed(0.5).range(5.0..=500.0));
                ui.label("H (mm):");
                ui.add(egui::DragValue::new(&mut elem.size_mm[1]).speed(0.5).range(5.0..=500.0));
            });
        });

        ui.add_space(4.0);
        ui.collapsing("Typography", |ui| {
            ui.horizontal(|ui| {
                ui.label("Font:");
                egui::ComboBox::from_id_salt("font_family_combo")
                    .selected_text(&elem.font_family)
                    .show_ui(ui, |ui| {
                        for font in available_fonts {
                            ui.selectable_value(&mut elem.font_family, font.clone(), font);
                        }
                    });
            });

            ui.horizontal(|ui| {
                ui.label("Size (pt):");
                ui.add(egui::DragValue::new(&mut elem.font_size_pt).speed(0.5).range(6.0..=144.0));
            });

            ui.horizontal(|ui| {
                ui.label("Alignment:");
                ui.selectable_value(&mut elem.alignment, TextAlignment::Left, "Left");
                ui.selectable_value(&mut elem.alignment, TextAlignment::Center, "Center");
                ui.selectable_value(&mut elem.alignment, TextAlignment::Right, "Right");
            });

            ui.horizontal(|ui| {
                ui.label("Color:");
                let mut color = Color32::from_rgba_unmultiplied(
                    elem.color_rgba[0],
                    elem.color_rgba[1],
                    elem.color_rgba[2],
                    elem.color_rgba[3],
                );
                if egui::color_picker::color_edit_button_srgba(
                    ui,
                    &mut color,
                    egui::color_picker::Alpha::Opaque,
                ).changed() {
                    elem.color_rgba = [color.r(), color.g(), color.b(), color.a()];
                }
            });
        });

        ui.add_space(4.0);
        ui.collapsing("Dynamic Binding (CSV)", |ui| {
            if !available_csv_columns.is_empty() {
                ui.label("Map to CSV Column:");
                let current_col = elem.field_key.clone().unwrap_or_else(|| "None".to_string());
                egui::ComboBox::from_id_salt("csv_col_picker")
                    .selected_text(&current_col)
                    .show_ui(ui, |ui| {
                        if ui.selectable_label(elem.field_key.is_none(), "None").clicked() {
                            elem.field_key = None;
                        }
                        for col in available_csv_columns {
                            let is_sel = elem.field_key.as_ref() == Some(col);
                            if ui.selectable_label(is_sel, col).clicked() {
                                elem.field_key = Some(col.clone());
                            }
                        }
                    });

                ui.add_space(2.0);
                ui.horizontal_wrapped(|ui| {
                    ui.label("Insert tag:");
                    for col in available_csv_columns {
                        let btn_label = format!("+ {{{{{}}}}}", col);
                        if ui.small_button(btn_label).clicked() {
                            elem.text.push_str(&format!(" {{{{{}}}}}", col));
                        }
                    }
                });
            } else {
                ui.label("Custom Placeholder Key:");
                let mut key = elem.field_key.clone().unwrap_or_default();
                if ui.text_edit_singleline(&mut key).changed() {
                    elem.field_key = if key.trim().is_empty() {
                        None
                    } else {
                        Some(key.trim().to_string())
                    };
                }
            }

            if let Some(ref k) = elem.field_key {
                ui.label(
                    egui::RichText::new(format!("Bound to {{{{{}}}}}", k))
                        .size(11.0)
                        .color(Color32::from_rgb(100, 180, 240)),
                );
            }
        });
    }
}
