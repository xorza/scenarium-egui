use anyhow::Result;
use eframe::{NativeOptions, egui};
use tracing_rolling_file::RollingFileAppenderBase;
use uuid::Uuid;

fn main() -> Result<()> {
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

#[derive(Debug)]
struct PlaygroundApp {
    value: f32,
    label: String,
    window_open: bool,
    last_generated_id: Uuid,
}

impl Default for PlaygroundApp {
    fn default() -> Self {
        Self {
            value: 42.0,
            label: String::new(),
            window_open: true,
            last_generated_id: Uuid::new_v4(),
        }
    }
}

impl eframe::App for PlaygroundApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.heading("egui playground");
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Drag the window by its title bar, and tweak the value.");
            ui.add(
                egui::DragValue::new(&mut self.value)
                    .speed(0.1)
                    .prefix("value: "),
            );
            ui.add_space(12.0);
            if ui.button("Generate GUID").clicked() {
                self.last_generated_id = Uuid::new_v4();
            }
            ui.label(format!("Current GUID: {}", self.last_generated_id));
        });

        egui::Window::new("Draggable Widget Window")
            .open(&mut self.window_open)
            .default_pos(egui::pos2(80.0, 120.0))
            .show(ctx, |ui| {
                ui.label("This window is draggable.");
                ui.add(egui::TextEdit::singleline(&mut self.label).hint_text("Type here"));
                ui.add_space(8.0);
                ui.add(egui::DragValue::new(&mut self.value).speed(0.5));
            });
    }
}
