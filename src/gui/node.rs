use eframe::egui;
use std::collections::HashMap;
use uuid::Uuid;

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

pub fn node_rect_for_graph(
    origin: egui::Pos2,
    node: &model::Node,
    scale: f32,
    layout: &NodeLayout,
    node_width: f32,
) -> egui::Rect {
    assert!(scale > 0.0, "graph scale must be positive");
    assert!(scale.is_finite(), "graph scale must be finite");
    layout.assert_valid();
    let node_size = node_size(node, layout, node_width);
    egui::Rect::from_min_size(origin + node.pos.to_vec2() * scale, node_size)
}

pub(crate) fn port_radius_for_scale(scale: f32) -> f32 {
    assert!(scale.is_finite(), "port scale must be finite");
    assert!(scale > 0.0, "port scale must be positive");
    let radius = (5.5 * scale).clamp(3.0, 7.5);
    assert!(radius.is_finite(), "port radius must be finite");
    assert!(radius > 0.0, "port radius must be positive");
    radius
}

pub fn render_nodes(
    ui: &mut egui::Ui,
    graph: &mut model::Graph,
    layout: &NodeLayout,
    node_widths: &HashMap<Uuid, f32>,
) {
    let rect = ui.available_rect_before_wrap();
    let painter = ui.painter_at(rect);
    let origin = rect.min + graph.pan;
    layout.assert_valid();
    assert!(graph.zoom > 0.0, "graph zoom must be positive");
    assert!(graph.zoom.is_finite(), "graph zoom must be finite");

    let visuals = ui.visuals();
    let text_color = visuals.text_color();
    let node_fill = visuals.widgets.noninteractive.bg_fill;
    let node_stroke = visuals.widgets.noninteractive.bg_stroke;
    let selected_stroke =
        egui::Stroke::new(node_stroke.width.max(2.0), visuals.selection.stroke.color);
    let input_port_color = egui::Color32::from_rgb(70, 150, 255);
    let output_port_color = egui::Color32::from_rgb(70, 200, 200);
    let input_hover_color = egui::Color32::from_rgb(120, 190, 255);
    let output_hover_color = egui::Color32::from_rgb(110, 230, 210);
    let heading_font = scaled_font(ui, egui::TextStyle::Heading, graph.zoom);
    let body_font = scaled_font(ui, egui::TextStyle::Body, graph.zoom);
    let header_text_offset = 4.0 * graph.zoom;
    let port_radius = port_radius_for_scale(graph.zoom);
    let mut selection_request = None;

    for node in &mut graph.nodes {
        let node_width = node_widths
            .get(&node.id)
            .copied()
            .expect("node width must be precomputed");
        let node_size = node_size(node, layout, node_width);
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

        for (index, _input) in node.inputs.iter().enumerate() {
            let center = node_input_pos(origin, node, index, layout, graph.zoom);

            let port_rect = egui::Rect::from_center_size(
                center,
                egui::vec2(port_radius * 2.0, port_radius * 2.0),
            );
            let color = if ui.rect_contains_pointer(port_rect) {
                input_hover_color
            } else {
                input_port_color
            };
            painter.circle_filled(center, port_radius, color);
        }

        for (index, _output) in node.outputs.iter().enumerate() {
            let center = node_output_pos(origin, node, index, layout, graph.zoom, node_width);

            let port_rect = egui::Rect::from_center_size(
                center,
                egui::vec2(port_radius * 2.0, port_radius * 2.0),
            );
            let color = if ui.rect_contains_pointer(port_rect) {
                output_hover_color
            } else {
                output_port_color
            };
            painter.circle_filled(center, port_radius, color);
        }

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
                    node_width - layout.padding,
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

fn node_size(node: &model::Node, layout: &NodeLayout, node_width: f32) -> egui::Vec2 {
    assert!(node_width.is_finite(), "node width must be finite");
    assert!(node_width > 0.0, "node width must be positive");
    let row_count = node.inputs.len().max(node.outputs.len()).max(1);
    let height = layout.header_height
        + layout.padding
        + layout.row_height * row_count as f32
        + layout.padding;
    egui::vec2(node_width, height)
}

pub(crate) fn node_input_pos(
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

pub(crate) fn node_output_pos(
    origin: egui::Pos2,
    node: &model::Node,
    index: usize,
    layout: &NodeLayout,
    scale: f32,
    node_width: f32,
) -> egui::Pos2 {
    assert!(
        index < node.outputs.len(),
        "output index must be within node outputs"
    );
    assert!(scale > 0.0, "graph scale must be positive");
    assert!(node_width.is_finite(), "node width must be finite");
    assert!(node_width > 0.0, "node width must be positive");
    let y = origin.y
        + node.pos.y * scale
        + layout.header_height
        + layout.padding
        + layout.row_height * index as f32
        + layout.row_height * 0.5;
    egui::pos2(origin.x + node.pos.x * scale + node_width, y)
}

pub(crate) fn bezier_control_offset(start: egui::Pos2, end: egui::Pos2, scale: f32) -> f32 {
    assert!(scale > 0.0, "graph scale must be positive");
    let dx = (end.x - start.x).abs();
    let offset = (dx * 0.5).max(40.0 * scale);
    assert!(offset.is_finite(), "bezier control offset must be finite");
    offset
}

pub(crate) fn compute_node_widths(
    painter: &egui::Painter,
    graph: &model::Graph,
    layout: &NodeLayout,
    heading_font: &egui::FontId,
    body_font: &egui::FontId,
    text_color: egui::Color32,
) -> HashMap<Uuid, f32> {
    layout.assert_valid();
    let mut widths = HashMap::with_capacity(graph.nodes.len());

    for node in &graph.nodes {
        let header_width =
            text_width(painter, heading_font, &node.name, text_color) + layout.padding * 2.0;

        let input_widths: Vec<f32> = node
            .inputs
            .iter()
            .map(|input| text_width(painter, body_font, &input.name, text_color))
            .collect();
        let output_widths: Vec<f32> = node
            .outputs
            .iter()
            .map(|output| text_width(painter, body_font, &output.name, text_color))
            .collect();

        let row_count = node.inputs.len().max(node.outputs.len()).max(1);
        let mut max_row_width: f32 = 0.0;

        let inter_side_padding = 0.0;
        for row in 0..row_count {
            let left = input_widths.get(row).copied().unwrap_or(0.0);
            let right = output_widths.get(row).copied().unwrap_or(0.0);
            let mut row_width = layout.padding * 2.0 + left + right;
            if left > 0.0 && right > 0.0 {
                row_width += inter_side_padding;
            }
            max_row_width = max_row_width.max(row_width);
        }

        let computed = layout.node_width.max(header_width.max(max_row_width));
        assert!(computed.is_finite(), "node width must be finite");
        assert!(computed > 0.0, "node width must be positive");
        let prior = widths.insert(node.id, computed);
        assert!(
            prior.is_none(),
            "node width map must not contain duplicate ids"
        );
    }

    widths
}

pub(crate) fn scaled_font(ui: &egui::Ui, style: egui::TextStyle, scale: f32) -> egui::FontId {
    assert!(scale.is_finite(), "font scale must be finite");
    assert!(scale > 0.0, "font scale must be positive");
    let base = style.resolve(ui.style());
    egui::FontId {
        size: base.size * scale,
        family: base.family.clone(),
    }
}

fn text_width(
    painter: &egui::Painter,
    font: &egui::FontId,
    text: &str,
    color: egui::Color32,
) -> f32 {
    let galley = painter.layout_no_wrap(text.to_string(), font.clone(), color);
    let width = galley.size().x;
    assert!(width.is_finite(), "text width must be finite");
    assert!(width >= 0.0, "text width must be non-negative");
    width
}
