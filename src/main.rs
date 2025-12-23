#![allow(dead_code)]
#![allow(unused_imports)]

mod gui;
mod init;
mod model;

use anyhow::Result;
use eframe::{NativeOptions, egui};
use std::ffi::OsStr;
use std::path::PathBuf;

fn main() -> Result<()> {
    init::init()?;

    let options = NativeOptions {
        renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    };

    eframe::run_native(
        "Scenarium",
        options,
        Box::new(|_cc| Ok(Box::new(ScenariumApp::default()))),
    )?;

    Ok(())
}

#[derive(Debug)]
struct ScenariumApp {
    graph: model::Graph,
    graph_path: PathBuf,
    last_status: Option<String>,
}

impl Default for ScenariumApp {
    fn default() -> Self {
        let graph = model::Graph::test_graph();
        graph
            .validate()
            .expect("sample graph should be valid for rendering");
        let graph_path = Self::default_graph_path();

        Self {
            graph,
            graph_path,
            last_status: None,
        }
    }
}

impl ScenariumApp {
    fn default_graph_path() -> PathBuf {
        let path = std::env::temp_dir().join("scenarium-graph.json");
        assert!(
            path.extension() == Some(OsStr::new("json")),
            "default graph path must use a .json extension"
        );
        path
    }

    fn set_status(&mut self, message: impl Into<String>) {
        self.last_status = Some(message.into());
    }

    fn set_graph(&mut self, graph: model::Graph, status: impl Into<String>) {
        graph
            .validate()
            .expect("graph should be valid before storing in app state");
        self.graph = graph;
        self.set_status(status);
    }

    fn new_graph(&mut self) {
        let graph = model::Graph::default();
        self.set_graph(graph, "Created new graph");
    }

    fn save_graph(&mut self) {
        assert!(
            self.graph_path.extension().is_some(),
            "graph save path must include a file extension"
        );
        match self.graph.serialize_to_file(&self.graph_path) {
            Ok(()) => self.set_status(format!("Saved graph to {}", self.graph_path.display())),
            Err(err) => self.set_status(format!("Save failed: {err}")),
        }
    }

    fn load_graph(&mut self) {
        assert!(
            self.graph_path.extension().is_some(),
            "graph load path must include a file extension"
        );
        match model::Graph::deserialize_from_file(&self.graph_path) {
            Ok(graph) => self.set_graph(
                graph,
                format!("Loaded graph from {}", self.graph_path.display()),
            ),
            Err(err) => self.set_status(format!("Load failed: {err}")),
        }
    }

    fn test_graph(&mut self) {
        let graph = model::Graph::test_graph();
        self.set_graph(graph, "Loaded sample test graph");
    }
}

impl eframe::App for ScenariumApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New").clicked() {
                        self.new_graph();
                        ui.close();
                    }
                    if ui.button("Save").clicked() {
                        self.save_graph();
                        ui.close();
                    }
                    if ui.button("Load").clicked() {
                        self.load_graph();
                        ui.close();
                    }
                    if ui.button("Test").clicked() {
                        self.test_graph();
                        ui.close();
                    }
                });
            });
        });

        egui::TopBottomPanel::bottom("status_panel").show(ctx, |ui| {
            if let Some(status) = self.last_status.as_deref() {
                ui.label(status);
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            gui::graph::render_graph(ui, &mut self.graph);
        });
    }
}
