use eframe::egui::{
    self, Color32, CursorIcon, FontFamily, FontId, Pos2, Rect, Sense, Stroke, TextureHandle,
    Ui, Vec2,
};
use uuid::Uuid;
use crate::editor::history::ProjectHistory;
use crate::editor::interactions::{CanvasTransform, InteractionMode, ResizeHandle};
use crate::model::element::TextAlignment;
use crate::model::project::CertificateProject;

pub struct CertificateCanvas;

impl CertificateCanvas {
    pub fn show(
        ui: &mut Ui,
        project: &mut CertificateProject,
        bg_texture: Option<&TextureHandle>,
        transform: &mut CanvasTransform,
        selected_element_id: &mut Option<Uuid>,
        interaction_mode: &mut InteractionMode,
        history: &mut ProjectHistory,
        active_record: Option<&std::collections::HashMap<String, String>>,
        status_hover_coord_mm: &mut Option<[f32; 2]>,
    ) {
        let (response, painter) = ui.allocate_painter(
            ui.available_size_before_wrap(),
            Sense::click_and_drag(),
        );
        let viewport = response.rect;
        let scale = transform.current_scale();

        // --- Handle Pan & Zoom ---
        // Mouse wheel zoom
        let scroll_delta = ui.input(|i| i.smooth_scroll_delta.y);
        if scroll_delta != 0.0 && response.hovered() {
            let zoom_factor = (1.0 + scroll_delta * 0.0015).clamp(0.85, 1.15);
            transform.zoom = (transform.zoom * zoom_factor).clamp(0.1, 5.0);
        }

        // Middle mouse button or Alt+drag pan
        if response.dragged_by(egui::PointerButton::Middle)
            || (response.dragged_by(egui::PointerButton::Primary) && ui.input(|i| i.modifiers.alt))
        {
            transform.pan += response.drag_delta();
            ui.ctx().set_cursor_icon(CursorIcon::Grabbing);
        }

        // Background canvas fill (blueprint dark workspace)
        painter.rect_filled(viewport, 0.0, Color32::from_rgb(26, 28, 33));

        // Draw subtle grid dots
        let grid_spacing = 30.0;
        let mut x = viewport.min.x + (transform.pan.x % grid_spacing);
        while x < viewport.max.x {
            let mut y = viewport.min.y + (transform.pan.y % grid_spacing);
            while y < viewport.max.y {
                painter.circle_filled(Pos2::new(x, y), 1.0, Color32::from_rgb(45, 48, 56));
                y += grid_spacing;
            }
            x += grid_spacing;
        }

        // Calculate document page rectangle
        let doc_rect = transform.compute_doc_rect(viewport, project.width_mm, project.height_mm);
        let doc_origin = doc_rect.min;

        // Hover coordinate tracking
        if let Some(hover_pos) = response.hover_pos() {
            if doc_rect.contains(hover_pos) {
                let mm = transform.screen_to_doc_mm(doc_origin, hover_pos);
                *status_hover_coord_mm = Some([mm[0].max(0.0), mm[1].max(0.0)]);
            } else {
                *status_hover_coord_mm = None;
            }
        } else {
            *status_hover_coord_mm = None;
        }

        // Draw shadow behind document
        let shadow_rect = doc_rect.translate(Vec2::new(4.0, 4.0));
        painter.rect_filled(shadow_rect, 2.0, Color32::from_black_alpha(80));

        // Draw document page background (white paper)
        painter.rect_filled(doc_rect, 0.0, Color32::WHITE);
        painter.rect_stroke(
            doc_rect,
            0.0,
            Stroke::new(1.0, Color32::from_gray(180)),
            egui::StrokeKind::Outside,
        );

        // Render certificate background image if loaded
        if let Some(texture) = bg_texture {
            painter.image(
                texture.id(),
                doc_rect,
                Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0)),
                Color32::WHITE,
            );
        } else {
            // Placeholder hint on blank certificate
            let center = doc_rect.center();
            painter.text(
                center,
                egui::Align2::CENTER_CENTER,
                "Certificate Canvas (Empty)\nClick 'Import Background' to load template image",
                FontId::proportional(15.0),
                Color32::from_gray(160),
            );
        }

        let mouse_pos = response.hover_pos();
        let is_primary_down = ui.input(|i| i.pointer.primary_down());
        let drag_delta = response.drag_delta();

        // 1. Process active drag / resize interaction
        if response.dragged_by(egui::PointerButton::Primary) && !ui.input(|i| i.modifiers.alt) && scale > 0.0 {
            match *interaction_mode {
                InteractionMode::DraggingElement(id) => {
                    if let Some(elem) = project.get_element_mut(&id) {
                        elem.pos_mm[0] += drag_delta.x / scale;
                        elem.pos_mm[1] += drag_delta.y / scale;
                        ui.ctx().set_cursor_icon(CursorIcon::Move);
                    }
                }
                InteractionMode::ResizingElement(id, handle) => {
                    if let Some(elem) = project.get_element_mut(&id) {
                        let dx = drag_delta.x / scale;
                        let dy = drag_delta.y / scale;

                        match handle {
                            ResizeHandle::Right => {
                                elem.size_mm[0] = (elem.size_mm[0] + dx).max(10.0);
                            }
                            ResizeHandle::Bottom => {
                                elem.size_mm[1] = (elem.size_mm[1] + dy).max(5.0);
                            }
                            ResizeHandle::Left => {
                                let new_w = (elem.size_mm[0] - dx).max(10.0);
                                elem.pos_mm[0] += elem.size_mm[0] - new_w;
                                elem.size_mm[0] = new_w;
                            }
                            ResizeHandle::Top => {
                                let new_h = (elem.size_mm[1] - dy).max(5.0);
                                elem.pos_mm[1] += elem.size_mm[1] - new_h;
                                elem.size_mm[1] = new_h;
                            }
                            ResizeHandle::BottomRight => {
                                elem.size_mm[0] = (elem.size_mm[0] + dx).max(10.0);
                                elem.size_mm[1] = (elem.size_mm[1] + dy).max(5.0);
                            }
                            ResizeHandle::BottomLeft => {
                                let new_w = (elem.size_mm[0] - dx).max(10.0);
                                elem.pos_mm[0] += elem.size_mm[0] - new_w;
                                elem.size_mm[0] = new_w;
                                elem.size_mm[1] = (elem.size_mm[1] + dy).max(5.0);
                            }
                            ResizeHandle::TopRight => {
                                elem.size_mm[0] = (elem.size_mm[0] + dx).max(10.0);
                                let new_h = (elem.size_mm[1] - dy).max(5.0);
                                elem.pos_mm[1] += elem.size_mm[1] - new_h;
                                elem.size_mm[1] = new_h;
                            }
                            ResizeHandle::TopLeft => {
                                let new_w = (elem.size_mm[0] - dx).max(10.0);
                                elem.pos_mm[0] += elem.size_mm[0] - new_w;
                                elem.size_mm[0] = new_w;
                                let new_h = (elem.size_mm[1] - dy).max(5.0);
                                elem.pos_mm[1] += elem.size_mm[1] - new_h;
                                elem.size_mm[1] = new_h;
                            }
                        }
                        ui.ctx().set_cursor_icon(handle.cursor_icon());
                    }
                }
                InteractionMode::None => {}
            }
        }

        // Finish drag / resize when mouse released
        if !is_primary_down && *interaction_mode != InteractionMode::None {
            *interaction_mode = InteractionMode::None;
            history.push_snapshot(project);
        }

        // 2. Render text elements & handles
        let mut hovered_handle = None;
        let mut element_clicked = None;
        let mut hovered_element_id = None;

        for element in &project.elements {
            if !element.is_visible {
                continue;
            }

            let elem_pos = transform.doc_mm_to_screen(doc_origin, element.pos_mm);
            let elem_size = transform.doc_size_to_screen(element.size_mm);
            let elem_rect = Rect::from_min_size(elem_pos, elem_size);

            let is_selected = *selected_element_id == Some(element.id);
            let is_hovered = mouse_pos.map(|p| elem_rect.contains(p)).unwrap_or(false);

            if is_hovered {
                hovered_element_id = Some(element.id);
            }

            if response.clicked() && is_hovered {
                element_clicked = Some(element.id);
            }

            // Element boundary highlights
            if is_selected {
                painter.rect_stroke(
                    elem_rect.expand(2.0),
                    2.0,
                    Stroke::new(1.5, Color32::from_rgb(0, 120, 215)),
                    egui::StrokeKind::Outside,
                );

                // Define 8 resize handles
                let handle_defs = [
                    (ResizeHandle::TopLeft, elem_rect.left_top()),
                    (ResizeHandle::Top, Pos2::new(elem_rect.center().x, elem_rect.top())),
                    (ResizeHandle::TopRight, elem_rect.right_top()),
                    (ResizeHandle::Right, Pos2::new(elem_rect.right(), elem_rect.center().y)),
                    (ResizeHandle::BottomRight, elem_rect.right_bottom()),
                    (ResizeHandle::Bottom, Pos2::new(elem_rect.center().x, elem_rect.bottom())),
                    (ResizeHandle::BottomLeft, elem_rect.left_bottom()),
                    (ResizeHandle::Left, Pos2::new(elem_rect.left(), elem_rect.center().y)),
                ];

                for (handle, pt) in &handle_defs {
                    let handle_rect = Rect::from_center_size(*pt, Vec2::splat(8.0));
                    let is_handle_hovered = mouse_pos.map(|p| handle_rect.contains(p)).unwrap_or(false);

                    if is_handle_hovered && *interaction_mode == InteractionMode::None {
                        hovered_handle = Some((element.id, *handle));
                        ui.ctx().set_cursor_icon(handle.cursor_icon());
                    }

                    // Draw handle square
                    let fill_color = if is_handle_hovered {
                        Color32::from_rgb(0, 150, 255)
                    } else {
                        Color32::WHITE
                    };
                    painter.rect_filled(handle_rect, 1.0, fill_color);
                    painter.rect_stroke(
                        handle_rect,
                        1.0,
                        Stroke::new(1.5, Color32::from_rgb(0, 120, 215)),
                        egui::StrokeKind::Outside,
                    );
                }
            } else if is_hovered {
                painter.rect_stroke(
                    elem_rect,
                    1.0,
                    Stroke::new(1.0, Color32::from_rgb(100, 160, 220)),
                    egui::StrokeKind::Outside,
                );
            }

            // Text rendering with selected font family
            let text_color = Color32::from_rgba_unmultiplied(
                element.color_rgba[0],
                element.color_rgba[1],
                element.color_rgba[2],
                element.color_rgba[3],
            );

            let screen_font_size = element.font_size_pt * (scale / 2.8346);
            let font_family = if element.font_family == "Default" {
                FontFamily::Proportional
            } else {
                FontFamily::Name(element.font_family.as_str().into())
            };
            let font_id = FontId::new(screen_font_size.max(6.0), font_family);

            let text_align = match element.alignment {
                TextAlignment::Left => egui::Align2::LEFT_CENTER,
                TextAlignment::Center => egui::Align2::CENTER_CENTER,
                TextAlignment::Right => egui::Align2::RIGHT_CENTER,
            };

            let anchor = match element.alignment {
                TextAlignment::Left => Pos2::new(elem_rect.left() + 4.0, elem_rect.center().y),
                TextAlignment::Center => elem_rect.center(),
                TextAlignment::Right => Pos2::new(elem_rect.right() - 4.0, elem_rect.center().y),
            };

            let display_text = if let Some(record) = active_record {
                crate::data::field_mapping::resolve_text_for_record(
                    &element.text,
                    element.field_key.as_deref(),
                    record,
                )
            } else {
                element.text.clone()
            };

            painter.text(anchor, text_align, &display_text, font_id, text_color);
        }

        // 3. Initiate drag / resize on mouse press
        if response.drag_started_by(egui::PointerButton::Primary) && !ui.input(|i| i.modifiers.alt) {
            history.push_snapshot(project);

            if let Some((elem_id, handle)) = hovered_handle {
                *interaction_mode = InteractionMode::ResizingElement(elem_id, handle);
            } else if let Some(elem_id) = hovered_element_id {
                *selected_element_id = Some(elem_id);
                *interaction_mode = InteractionMode::DraggingElement(elem_id);
            }
        }

        // Selection updates
        if let Some(id) = element_clicked {
            *selected_element_id = Some(id);
        } else if response.clicked() && hovered_handle.is_none() && hovered_element_id.is_none() {
            *selected_element_id = None;
        }
    }
}
