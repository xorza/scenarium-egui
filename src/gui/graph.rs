use eframe::egui;

use crate::{gui::node, model};

const MIN_ZOOM: f32 = 0.2;
const MAX_ZOOM: f32 = 4.0;

pub fn render_graph(ui: &mut egui::Ui, graph: &mut model::Graph) {
    let rect = ui.available_rect_before_wrap();
    let painter = ui.painter_at(rect);
    assert!(graph.zoom.is_finite(), "graph zoom must be finite");
    assert!(graph.zoom > 0.0, "graph zoom must be positive");

    let pan_id = ui.make_persistent_id("graph_pan");
    let pan_response = ui.interact(rect, pan_id, egui::Sense::drag());

    if pan_response.dragged() {
        graph.pan += pan_response.drag_delta();
    }

    let pointer_pos = ui.input(|input| input.pointer.hover_pos());
    let zoom_active = pointer_pos.map(|pos| rect.contains(pos)).unwrap_or(false);

    if zoom_active {
        let zoom_delta = ui.input(|input| input.zoom_delta());
        if (zoom_delta - 1.0).abs() > f32::EPSILON {
            let clamped_zoom = (graph.zoom * zoom_delta).clamp(MIN_ZOOM, MAX_ZOOM);
            assert!(clamped_zoom.is_finite(), "clamped zoom must be finite");

            if (clamped_zoom - graph.zoom).abs() > f32::EPSILON {
                let cursor = pointer_pos.unwrap_or_else(|| rect.center());
                let origin = rect.min;
                let graph_pos = (cursor - origin - graph.pan) / graph.zoom;

                graph.zoom = clamped_zoom;
                graph.pan = cursor - origin - graph_pos * graph.zoom;
            }
        }
    }

    draw_dotted_background(&painter, rect, graph);
    node::render_graph(ui, graph);
}

fn draw_dotted_background(painter: &egui::Painter, rect: egui::Rect, graph: &model::Graph) {
    let base_spacing = 24.0;
    let spacing = base_spacing * graph.zoom;
    let radius = (1.2 * graph.zoom).clamp(0.6, 2.4);
    let color = egui::Color32::from_rgba_unmultiplied(255, 255, 255, 28);

    assert!(spacing.is_finite(), "dot spacing must be finite");
    assert!(spacing > 0.0, "dot spacing must be positive");
    assert!(radius.is_finite(), "dot radius must be finite");
    assert!(radius > 0.0, "dot radius must be positive");

    let origin = rect.min + graph.pan;
    let offset_x = (rect.left() - origin.x).rem_euclid(spacing);
    let offset_y = (rect.top() - origin.y).rem_euclid(spacing);
    let start_x = rect.left() - offset_x - spacing;
    let start_y = rect.top() - offset_y - spacing;

    let mut y = start_y;
    while y <= rect.bottom() + spacing {
        let mut x = start_x;
        while x <= rect.right() + spacing {
            painter.circle_filled(egui::pos2(x, y), radius, color);
            x += spacing;
        }
        y += spacing;
    }
}
