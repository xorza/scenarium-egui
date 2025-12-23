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
    pub(crate) fn assert_valid(&self) {
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

    pub(crate) fn scaled(&self, scale: f32) -> Self {
        assert!(scale > 0.0, "layout scale must be positive");
        assert!(scale.is_finite(), "layout scale must be finite");

        Self {
            node_width: self.node_width * scale,
            header_height: self.header_height * scale,
            row_height: self.row_height * scale,
            padding: self.padding * scale,
            corner_radius: self.corner_radius * scale,
        }
    }
}

pub fn node_rect_for_graph(origin: egui::Pos2, node: &model::Node, scale: f32) -> egui::Rect {
    assert!(scale > 0.0, "graph scale must be positive");
    assert!(scale.is_finite(), "graph scale must be finite");
    let layout = NodeLayout::default().scaled(scale);
    layout.assert_valid();
    let node_size = node_size(node, &layout);
    egui::Rect::from_min_size(origin + node.pos.to_vec2() * scale, node_size)
}

pub fn render_graph(ui: &mut egui::Ui, graph: &mut model::Graph) {
    let rect = ui.available_rect_before_wrap();
    let painter = ui.painter_at(rect);
    let origin = rect.min + graph.pan;
    let layout = NodeLayout::default().scaled(graph.zoom);

    layout.assert_valid();
    assert!(graph.zoom > 0.0, "graph zoom must be positive");
    assert!(graph.zoom.is_finite(), "graph zoom must be finite");

    {
        let node_lookup: HashMap<_, _> = graph.nodes.iter().map(|node| (node.id, node)).collect();

        for node in &graph.nodes {
            for (input_index, input) in node.inputs.iter().enumerate() {
                let Some(connection) = &input.connection else {
                    continue;
                };

                let source_node = node_lookup
                    .get(&connection.node_id)
                    .expect("graph validation must guarantee source nodes exist");

                let start = node_output_pos(
                    origin,
                    source_node,
                    connection.output_index,
                    &layout,
                    graph.zoom,
                );
                let end = node_input_pos(origin, node, input_index, &layout, graph.zoom);

                let stroke = egui::Stroke::new(2.0, egui::Color32::from_rgb(80, 160, 255));
                let control_offset = bezier_control_offset(start, end, graph.zoom);
                let curve = egui::epaint::CubicBezierShape::from_points_stroke(
                    [
                        start,
                        start + egui::vec2(control_offset, 0.0),
                        end + egui::vec2(-control_offset, 0.0),
                        end,
                    ],
                    false,
                    egui::Color32::TRANSPARENT,
                    stroke,
                );
                painter.add(curve);
            }
        }
    }

    let visuals = ui.visuals();
    let text_color = visuals.text_color();
    let node_fill = visuals.widgets.noninteractive.bg_fill;
    let node_stroke = visuals.widgets.noninteractive.bg_stroke;
    let selected_stroke =
        egui::Stroke::new(node_stroke.width.max(2.0), visuals.selection.stroke.color);
    let heading_font = scaled_font(ui, egui::TextStyle::Heading, graph.zoom);
    let body_font = scaled_font(ui, egui::TextStyle::Body, graph.zoom);
    let header_text_offset = 4.0 * graph.zoom;
    let mut selection_request = None;

    for node in &mut graph.nodes {
        let node_size = node_size(node, &layout);
        let node_rect =
            egui::Rect::from_min_size(origin + node.pos.to_vec2() * graph.zoom, node_size);
        let header_rect =
            egui::Rect::from_min_size(node_rect.min, egui::vec2(node_size.x, layout.header_height));

        let node_id = ui.make_persistent_id(("node_body", node.id));
        let body_response = ui.interact(node_rect, node_id, egui::Sense::click());

        let header_id = ui.make_persistent_id(("node_header", node.id));
        let response = ui.interact(header_rect, header_id, egui::Sense::drag());

        if response.dragged() {
            node.pos += response.drag_delta() / graph.zoom;
        }

        if response.clicked() || response.dragged() || body_response.clicked() {
            selection_request = Some(node.id);
        }

        let selected_id = selection_request.or(graph.selected_node_id);
        let is_selected = selected_id.is_some_and(|id| id == node.id);

        painter.rect(
            node_rect,
            layout.corner_radius,
            node_fill,
            if is_selected {
                selected_stroke
            } else {
                node_stroke
            },
            egui::StrokeKind::Inside,
        );

        painter.text(
            node_rect.min + egui::vec2(layout.padding, header_text_offset),
            egui::Align2::LEFT_TOP,
            &node.name,
            heading_font.clone(),
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
                body_font.clone(),
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
                body_font.clone(),
                text_color,
            );
        }
    }

    if let Some(selected_id) = selection_request {
        graph.select_node(selected_id);
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
    scale: f32,
) -> egui::Pos2 {
    assert!(
        index < node.inputs.len(),
        "input index must be within node inputs"
    );
    assert!(scale > 0.0, "graph scale must be positive");
    let y = origin.y
        + node.pos.y * scale
        + layout.header_height
        + layout.padding
        + layout.row_height * index as f32
        + layout.row_height * 0.5;
    egui::pos2(origin.x + node.pos.x * scale, y)
}

fn node_output_pos(
    origin: egui::Pos2,
    node: &model::Node,
    index: usize,
    layout: &NodeLayout,
    scale: f32,
) -> egui::Pos2 {
    assert!(
        index < node.outputs.len(),
        "output index must be within node outputs"
    );
    assert!(scale > 0.0, "graph scale must be positive");
    let y = origin.y
        + node.pos.y * scale
        + layout.header_height
        + layout.padding
        + layout.row_height * index as f32
        + layout.row_height * 0.5;
    egui::pos2(origin.x + node.pos.x * scale + layout.node_width, y)
}

fn bezier_control_offset(start: egui::Pos2, end: egui::Pos2, scale: f32) -> f32 {
    assert!(scale > 0.0, "graph scale must be positive");
    let dx = (end.x - start.x).abs();
    let offset = (dx * 0.5).max(40.0 * scale);
    assert!(offset.is_finite(), "bezier control offset must be finite");
    offset
}

fn scaled_font(ui: &egui::Ui, style: egui::TextStyle, scale: f32) -> egui::FontId {
    assert!(scale.is_finite(), "font scale must be finite");
    assert!(scale > 0.0, "font scale must be positive");
    let base = style.resolve(ui.style());
    egui::FontId {
        size: base.size * scale,
        family: base.family.clone(),
    }
}
