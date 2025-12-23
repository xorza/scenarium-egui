use eframe::egui;

use crate::{gui::node, model};

pub fn render_graph(ui: &mut egui::Ui, graph: &mut model::Graph) {
    let rect = ui.available_rect_before_wrap();
    let painter = ui.painter_at(rect);

    draw_dotted_background(&painter, rect);

    node::render_graph(ui, graph);
}

fn draw_dotted_background(painter: &egui::Painter, rect: egui::Rect) {
    let spacing = 16.0;
    let radius = 1.2;
    let color = egui::Color32::from_rgba_unmultiplied(255, 255, 255, 28);

    assert!(spacing > 0.0, "dot spacing must be positive");
    assert!(radius > 0.0, "dot radius must be positive");

    let start_x = rect.left() - (rect.left() % spacing) - spacing;
    let start_y = rect.top() - (rect.top() % spacing) - spacing;

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
