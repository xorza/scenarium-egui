#![allow(dead_code)]
#![allow(unused_imports)]

mod model;

use anyhow::Result;
use eframe::{NativeOptions, egui};
use std::collections::HashMap;
use tracing_rolling_file::RollingFileAppenderBase;

fn main() -> Result<()> {
    dotenv::dotenv().ok();
    init_trace().ok();

    let options = NativeOptions {
        renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    };

    eframe::run_native(
        "Scenarium",
        options,
        Box::new(|_cc| Ok(Box::new(PlaygroundApp::default()))),
    )?;

    Ok(())
}

fn init_trace() -> Result<()> {
    std::fs::create_dir_all("log")?;
    let appender = RollingFileAppenderBase::builder()
        .filename("log/egui-playground.log".to_string())
        .max_filecount(10)
        .condition_max_file_size(10 * 1024 * 1024)
        .build()
        .expect("failed to initialize log appender");
    let (non_blocking, _log_guard) = appender.get_non_blocking_appender();
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .with_writer(non_blocking)
        .init();

    Ok(())
}

#[derive(Debug)]
struct PlaygroundApp {
    graph: model::Graph,
}

impl Default for PlaygroundApp {
    fn default() -> Self {
        let graph = model::Graph::test_graph();
        graph
            .validate()
            .expect("sample graph should be valid for rendering");

        Self { graph }
    }
}

impl eframe::App for PlaygroundApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.heading("Scenarium");
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let graph = &self.graph;
            let rect = ui.available_rect_before_wrap();
            let painter = ui.painter_at(rect);
            let origin = rect.min;

            let node_width = 180.0;
            let header_height = 22.0;
            let row_height = 18.0;
            let padding = 8.0;

            let node_lookup: HashMap<_, _> =
                graph.nodes.iter().map(|node| (node.id, node)).collect();

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
                        node_width,
                        header_height,
                        row_height,
                        padding,
                    );
                    let end = node_input_pos(
                        origin,
                        node,
                        input_index,
                        header_height,
                        row_height,
                        padding,
                    );

                    painter.line_segment(
                        [start, end],
                        egui::Stroke::new(2.0, egui::Color32::from_rgb(80, 160, 255)),
                    );
                }
            }

            for node in &graph.nodes {
                let node_size = node_size(node, node_width, header_height, row_height, padding);
                let node_rect = egui::Rect::from_min_size(origin + node.pos.to_vec2(), node_size);

                painter.rect(
                    node_rect,
                    6.0,
                    ui.visuals().widgets.noninteractive.bg_fill,
                    ui.visuals().widgets.noninteractive.bg_stroke,
                    egui::StrokeKind::Inside,
                );

                painter.text(
                    node_rect.min + egui::vec2(padding, 4.0),
                    egui::Align2::LEFT_TOP,
                    &node.name,
                    egui::TextStyle::Heading.resolve(ui.style()),
                    ui.visuals().text_color(),
                );

                for (index, input) in node.inputs.iter().enumerate() {
                    let text_pos = node_rect.min
                        + egui::vec2(padding, header_height + padding + row_height * index as f32);
                    painter.text(
                        text_pos,
                        egui::Align2::LEFT_TOP,
                        &input.name,
                        egui::TextStyle::Body.resolve(ui.style()),
                        ui.visuals().text_color(),
                    );
                }

                for (index, output) in node.outputs.iter().enumerate() {
                    let text_pos = node_rect.min
                        + egui::vec2(
                            node_width - padding,
                            header_height + padding + row_height * index as f32,
                        );
                    painter.text(
                        text_pos,
                        egui::Align2::RIGHT_TOP,
                        &output.name,
                        egui::TextStyle::Body.resolve(ui.style()),
                        ui.visuals().text_color(),
                    );
                }
            }
        });
    }
}

fn node_size(
    node: &model::Node,
    node_width: f32,
    header_height: f32,
    row_height: f32,
    padding: f32,
) -> egui::Vec2 {
    let row_count = node.inputs.len().max(node.outputs.len()).max(1);
    assert!(node_width > 0.0, "node width must be positive");
    assert!(header_height >= 0.0, "header height must be non-negative");
    assert!(row_height > 0.0, "row height must be positive");
    assert!(padding >= 0.0, "padding must be non-negative");
    let height = header_height + padding + row_height * row_count as f32 + padding;
    egui::vec2(node_width, height)
}

fn node_input_pos(
    origin: egui::Pos2,
    node: &model::Node,
    index: usize,
    header_height: f32,
    row_height: f32,
    padding: f32,
) -> egui::Pos2 {
    assert!(
        index < node.inputs.len(),
        "input index must be within node inputs"
    );
    let y = origin.y
        + node.pos.y
        + header_height
        + padding
        + row_height * index as f32
        + row_height * 0.5;
    egui::pos2(origin.x + node.pos.x, y)
}

fn node_output_pos(
    origin: egui::Pos2,
    node: &model::Node,
    index: usize,
    node_width: f32,
    header_height: f32,
    row_height: f32,
    padding: f32,
) -> egui::Pos2 {
    assert!(
        index < node.outputs.len(),
        "output index must be within node outputs"
    );
    let y = origin.y
        + node.pos.y
        + header_height
        + padding
        + row_height * index as f32
        + row_height * 0.5;
    egui::pos2(origin.x + node.pos.x + node_width, y)
}
