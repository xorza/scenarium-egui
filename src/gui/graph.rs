use eframe::egui;

use crate::{gui::node, model};
use std::collections::HashSet;
use uuid::Uuid;

const MIN_ZOOM: f32 = 0.2;
const MAX_ZOOM: f32 = 4.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ConnectionKey {
    target_node_id: Uuid,
    input_index: usize,
}

#[derive(Debug, Default)]
pub struct ConnectionBreaker {
    pub active: bool,
    pub points: Vec<egui::Pos2>,
}

impl ConnectionBreaker {
    pub fn reset(&mut self) {
        self.active = false;
        self.points.clear();
    }
}

pub fn render_graph(ui: &mut egui::Ui, graph: &mut model::Graph, breaker: &mut ConnectionBreaker) {
    let rect = ui.available_rect_before_wrap();
    let painter = ui.painter_at(rect);
    assert!(graph.zoom.is_finite(), "graph zoom must be finite");
    assert!(graph.zoom > 0.0, "graph zoom must be positive");

    let pointer_pos = ui.input(|input| input.pointer.hover_pos());
    let pointer_in_rect = pointer_pos.map(|pos| rect.contains(pos)).unwrap_or(false);
    let origin = rect.min + graph.pan;
    let port_offset = port_offset_for_scale(graph.zoom);
    let pointer_over_node = pointer_pos
        .filter(|pos| rect.contains(*pos))
        .is_some_and(|pos| {
            graph.nodes.iter().any(|node| {
                let node_rect = node::node_rect_for_graph(origin, node, graph.zoom);
                node_rect
                    .expand2(egui::vec2(port_offset, 0.0))
                    .contains(pos)
            })
        });
    let pan_id = ui.make_persistent_id("graph_pan");
    let pan_response = ui.interact(
        rect,
        pan_id,
        if breaker.active || pointer_over_node {
            egui::Sense::hover()
        } else {
            egui::Sense::drag()
        },
    );

    if pan_response.dragged() && !pointer_over_node && !breaker.active {
        graph.pan += pan_response.drag_delta();
    }

    let primary_pressed = ui.input(|input| input.pointer.primary_pressed());
    let primary_down = ui.input(|input| input.pointer.primary_down());
    let primary_released = ui.input(|input| input.pointer.primary_released());

    if !breaker.active && primary_pressed && pointer_in_rect && !pointer_over_node {
        breaker.active = true;
        breaker.points.clear();
        if let Some(pos) = pointer_pos {
            breaker.points.push(pos);
        }
    }

    if breaker.active
        && primary_down
        && let Some(pos) = pointer_pos
    {
        let should_add = breaker
            .points
            .last()
            .map(|last| last.distance(pos) > 2.0)
            .unwrap_or(true);
        if should_add {
            breaker.points.push(pos);
        }
    }

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

    let layout = node::NodeLayout::default().scaled(graph.zoom);
    layout.assert_valid();
    let curves = collect_connection_curves(graph, origin, &layout);
    let highlighted = if breaker.active && breaker.points.len() > 1 {
        connection_hits(&curves, &breaker.points)
    } else {
        HashSet::new()
    };
    draw_connections(&painter, &curves, &highlighted);

    if breaker.active && breaker.points.len() > 1 {
        let breaker_stroke = egui::Stroke::new(2.5, egui::Color32::from_rgb(255, 120, 120));
        painter.add(egui::Shape::line(breaker.points.clone(), breaker_stroke));
    }

    node::render_nodes(ui, graph);

    if breaker.active && primary_released {
        remove_connections(graph, &highlighted);
        breaker.reset();
    }
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

#[derive(Debug, Clone)]
struct ConnectionCurve {
    key: ConnectionKey,
    start: egui::Pos2,
    end: egui::Pos2,
    control_offset: f32,
}

fn collect_connection_curves(
    graph: &model::Graph,
    origin: egui::Pos2,
    layout: &node::NodeLayout,
) -> Vec<ConnectionCurve> {
    let node_lookup: std::collections::HashMap<_, _> =
        graph.nodes.iter().map(|node| (node.id, node)).collect();
    let mut curves = Vec::new();

    for node in &graph.nodes {
        for (input_index, input) in node.inputs.iter().enumerate() {
            let Some(connection) = &input.connection else {
                continue;
            };
            let source_node = node_lookup
                .get(&connection.node_id)
                .expect("graph validation must guarantee source nodes exist");
            let start = node::node_output_pos(
                origin,
                source_node,
                connection.output_index,
                layout,
                graph.zoom,
            );
            let end = node::node_input_pos(origin, node, input_index, layout, graph.zoom);
            let control_offset = node::bezier_control_offset(start, end, graph.zoom);
            curves.push(ConnectionCurve {
                key: ConnectionKey {
                    target_node_id: node.id,
                    input_index,
                },
                start,
                end,
                control_offset,
            });
        }
    }

    curves
}

fn draw_connections(
    painter: &egui::Painter,
    curves: &[ConnectionCurve],
    highlighted: &HashSet<ConnectionKey>,
) {
    let base_stroke = egui::Stroke::new(2.0, egui::Color32::from_rgb(80, 160, 255));
    let highlight_stroke = egui::Stroke::new(2.5, egui::Color32::from_rgb(255, 90, 90));

    for curve in curves {
        let stroke = if highlighted.contains(&curve.key) {
            highlight_stroke
        } else {
            base_stroke
        };
        let control_offset = curve.control_offset;
        let shape = egui::epaint::CubicBezierShape::from_points_stroke(
            [
                curve.start,
                curve.start + egui::vec2(control_offset, 0.0),
                curve.end + egui::vec2(-control_offset, 0.0),
                curve.end,
            ],
            false,
            egui::Color32::TRANSPARENT,
            stroke,
        );
        painter.add(shape);
    }
}

fn connection_hits(curves: &[ConnectionCurve], breaker: &[egui::Pos2]) -> HashSet<ConnectionKey> {
    let mut hits = HashSet::new();
    let breaker_segments = breaker.windows(2).map(|pair| (pair[0], pair[1]));

    for curve in curves {
        let samples = sample_cubic_bezier(
            curve.start,
            curve.start + egui::vec2(curve.control_offset, 0.0),
            curve.end + egui::vec2(-curve.control_offset, 0.0),
            curve.end,
            24,
        );
        let curve_segments = samples.windows(2).map(|pair| (pair[0], pair[1]));
        let mut hit = false;
        for (a1, a2) in breaker_segments.clone() {
            for (b1, b2) in curve_segments.clone() {
                if segments_intersect(a1, a2, b1, b2) {
                    hit = true;
                    break;
                }
            }
            if hit {
                break;
            }
        }
        if hit {
            hits.insert(curve.key);
        }
    }

    hits
}

fn sample_cubic_bezier(
    p0: egui::Pos2,
    p1: egui::Pos2,
    p2: egui::Pos2,
    p3: egui::Pos2,
    steps: usize,
) -> Vec<egui::Pos2> {
    assert!(steps >= 2, "bezier sampling steps must be at least 2");
    let mut points = Vec::with_capacity(steps + 1);
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let one_minus = 1.0 - t;
        let a = one_minus * one_minus * one_minus;
        let b = 3.0 * one_minus * one_minus * t;
        let c = 3.0 * one_minus * t * t;
        let d = t * t * t;
        let x = a * p0.x + b * p1.x + c * p2.x + d * p3.x;
        let y = a * p0.y + b * p1.y + c * p2.y + d * p3.y;
        points.push(egui::pos2(x, y));
    }
    points
}

fn segments_intersect(a1: egui::Pos2, a2: egui::Pos2, b1: egui::Pos2, b2: egui::Pos2) -> bool {
    let o1 = orient(a1, a2, b1);
    let o2 = orient(a1, a2, b2);
    let o3 = orient(b1, b2, a1);
    let o4 = orient(b1, b2, a2);
    let eps = 1e-6;

    if o1.abs() < eps && on_segment(a1, a2, b1) {
        return true;
    }
    if o2.abs() < eps && on_segment(a1, a2, b2) {
        return true;
    }
    if o3.abs() < eps && on_segment(b1, b2, a1) {
        return true;
    }
    if o4.abs() < eps && on_segment(b1, b2, a2) {
        return true;
    }

    (o1 > 0.0) != (o2 > 0.0) && (o3 > 0.0) != (o4 > 0.0)
}

fn orient(a: egui::Pos2, b: egui::Pos2, c: egui::Pos2) -> f32 {
    (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x)
}

fn on_segment(a: egui::Pos2, b: egui::Pos2, p: egui::Pos2) -> bool {
    let min_x = a.x.min(b.x);
    let max_x = a.x.max(b.x);
    let min_y = a.y.min(b.y);
    let max_y = a.y.max(b.y);
    p.x >= min_x - 1e-6 && p.x <= max_x + 1e-6 && p.y >= min_y - 1e-6 && p.y <= max_y + 1e-6
}

fn remove_connections(graph: &mut model::Graph, highlighted: &HashSet<ConnectionKey>) {
    if highlighted.is_empty() {
        return;
    }
    for node in &mut graph.nodes {
        for (input_index, input) in node.inputs.iter_mut().enumerate() {
            let key = ConnectionKey {
                target_node_id: node.id,
                input_index,
            };
            if highlighted.contains(&key) {
                input.connection = None;
            }
        }
    }
}

fn port_offset_for_scale(scale: f32) -> f32 {
    let radius = (5.5 * scale).clamp(3.0, 7.5);
    radius * 0.8
}
