#![allow(dead_code)]
#![allow(unused_imports)]

mod gui;
mod init;
mod model;

use anyhow::Result;
use eframe::{NativeOptions, egui};

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
}

impl Default for ScenariumApp {
    fn default() -> Self {
        let graph = model::Graph::test_graph();
        graph
            .validate()
            .expect("sample graph should be valid for rendering");

        Self { graph }
    }
}

impl eframe::App for ScenariumApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.heading("Scenarium");
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            gui::node::render_graph(ui, &self.graph);
        });
    }
}
