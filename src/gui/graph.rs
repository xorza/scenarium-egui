use eframe::egui;

use crate::{gui::node, model};

pub fn render_graph(ui: &mut egui::Ui, graph: &mut model::Graph) {
    let rect = ui.available_rect_before_wrap();
    let visuals = ui.visuals();
    let painter = ui.painter_at(rect);

    painter.rect_filled(rect, 0.0, visuals.extreme_bg_color);

    node::render_graph(ui, graph);
}
