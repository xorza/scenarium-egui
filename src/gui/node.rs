use std::collections::HashMap;

use eframe::egui;

use crate::model;

#[derive(Debug)]
pub struct NodeLayout {
    pub node_width: f32,
    pub header_height: f32,
    pub row_height: f32,
    pub padding: f32,
    pub corner_radius: f32,
}

impl Default for NodeLayout {
    fn default() -> Self {
        Self {
            node_width: 180.0,
            header_height: 22.0,
            row_height: 18.0,
            padding: 8.0,
            corner_radius: 6.0,
        }
    }
}

impl NodeLayout {
    fn assert_valid(&self) {
        assert!(self.node_width > 0.0, "node width must be positive");
        assert!(
            self.header_height >= 0.0,
            "header height must be non-negative"
        );
        assert!(self.row_height > 0.0, "row height must be positive");
        assert!(self.padding >= 0.0, "padding must be non-negative");
        assert!(
            self.corner_radius >= 0.0,
            "corner radius must be non-negative"
        );
    }
}

pub fn render_graph(ui: &mut egui::Ui, graph: &model::Graph) {
    let rect = ui.available_rect_before_wrap();
    let painter = ui.painter_at(rect);
    let origin = rect.min;
    let layout = NodeLayout::default();

    layout.assert_valid();

    let node_lookup: HashMap<_, _> = graph.nodes.iter().map(|node| (node.id, node)).collect();

    for node in &graph.nodes {
        for (input_index, input) in node.inputs.iter().enumerate() {
            let Some(connection) = &input.connection else {
                continue;
            };

            let source_node = node_lookup
                .get(&connection.node_id)
                .expect("graph validation must guarantee source nodes exist");

            let start = node_output_pos(origin, source_node, connection.output_index, &layout);
            let end = node_input_pos(origin, node, input_index, &layout);

            painter.line_segment(
                [start, end],
                egui::Stroke::new(2.0, egui::Color32::from_rgb(80, 160, 255)),
            );
        }
    }

    let visuals = ui.visuals();
    let text_color = visuals.text_color();
    let node_fill = visuals.widgets.noninteractive.bg_fill;
    let node_stroke = visuals.widgets.noninteractive.bg_stroke;

    for node in &graph.nodes {
        let node_size = node_size(node, &layout);
        let node_rect = egui::Rect::from_min_size(origin + node.pos.to_vec2(), node_size);

        painter.rect(
            node_rect,
            layout.corner_radius,
            node_fill,
            node_stroke,
            egui::StrokeKind::Inside,
        );

        painter.text(
            node_rect.min + egui::vec2(layout.padding, 4.0),
            egui::Align2::LEFT_TOP,
            &node.name,
            egui::TextStyle::Heading.resolve(ui.style()),
            text_color,
        );

        for (index, input) in node.inputs.iter().enumerate() {
            let text_pos = node_rect.min
                + egui::vec2(
                    layout.padding,
                    layout.header_height + layout.padding + layout.row_height * index as f32,
                );
            painter.text(
                text_pos,
                egui::Align2::LEFT_TOP,
                &input.name,
                egui::TextStyle::Body.resolve(ui.style()),
                text_color,
            );
        }

        for (index, output) in node.outputs.iter().enumerate() {
            let text_pos = node_rect.min
                + egui::vec2(
                    layout.node_width - layout.padding,
                    layout.header_height + layout.padding + layout.row_height * index as f32,
                );
            painter.text(
                text_pos,
                egui::Align2::RIGHT_TOP,
                &output.name,
                egui::TextStyle::Body.resolve(ui.style()),
                text_color,
            );
        }
    }
}

fn node_size(node: &model::Node, layout: &NodeLayout) -> egui::Vec2 {
    let row_count = node.inputs.len().max(node.outputs.len()).max(1);
    let height = layout.header_height
        + layout.padding
        + layout.row_height * row_count as f32
        + layout.padding;
    egui::vec2(layout.node_width, height)
}

fn node_input_pos(
    origin: egui::Pos2,
    node: &model::Node,
    index: usize,
    layout: &NodeLayout,
) -> egui::Pos2 {
    assert!(
        index < node.inputs.len(),
        "input index must be within node inputs"
    );
    let y = origin.y
        + node.pos.y
        + layout.header_height
        + layout.padding
        + layout.row_height * index as f32
        + layout.row_height * 0.5;
    egui::pos2(origin.x + node.pos.x, y)
}

fn node_output_pos(
    origin: egui::Pos2,
    node: &model::Node,
    index: usize,
    layout: &NodeLayout,
) -> egui::Pos2 {
    assert!(
        index < node.outputs.len(),
        "output index must be within node outputs"
    );
    let y = origin.y
        + node.pos.y
        + layout.header_height
        + layout.padding
        + layout.row_height * index as f32
        + layout.row_height * 0.5;
    egui::pos2(origin.x + node.pos.x + layout.node_width, y)
}
