#![allow(dead_code)]
#![allow(unused_imports)]

mod model;

use anyhow::Result;
use eframe::{NativeOptions, egui};
use tracing_rolling_file::RollingFileAppenderBase;
use uuid::Uuid;

fn main() -> Result<()> {
    dotenv::dotenv().ok();
    init_trace().ok();

    let options = NativeOptions {
        renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    };

    eframe::run_native(
        "egui playground",
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

#[derive(Debug, Default)]
struct PlaygroundApp {
    graph: Vec<model::Graph>,
}

impl eframe::App for PlaygroundApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        unimplemented!();
    }
}
