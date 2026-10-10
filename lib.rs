//! VectorCraft for Android tablets: the same egui app as the desktop build, started through
//! eframe's Android runner (NativeActivity). Stage 1: it starts, draws and takes touch input.
//! File pickers, clipboard and preferences come in the next stages.
#![cfg(target_os = "android")]

use vectorcraft_engine::Session;
use vectorcraft_engine::cmd::fileio;
use vectorcraft_engine::file_access;
use vectorcraft_ui_egui::{Services, VectorcraftApp};
use winit::platform::android::activity::AndroidApp;

struct App {
    app: VectorcraftApp,
}

impl eframe::App for App {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        file_access::unconfined(|| {
            self.app.logic(ctx);
            if self.app.ui.status == "quit" {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        });
    }

    fn raw_input_hook(&mut self, _ctx: &egui::Context, raw: &mut egui::RawInput) {
        file_access::unconfined(|| self.app.raw_input_hook(raw));
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        file_access::unconfined(|| self.app.ui(ui));
    }

    fn on_exit(&mut self) {}
}

/// Write a file the safe way: a failed write keeps the old file.
fn write_file(path: &str, bytes: &[u8]) -> Result<(), String> {
    fileio::write_atomic(std::path::Path::new(path), bytes).map_err(|e| e.to_string())
}

fn services() -> Services {
    Services {
        read: Some(Box::new(|p: &str| std::fs::read(p).map_err(|e| e.to_string()))),
        write: Some(Box::new(write_file)),
        write_shared: Some(std::sync::Arc::new(write_file)),
        ..Default::default()
    }
}

#[unsafe(no_mangle)]
fn android_main(android: AndroidApp) {
    android_logger::init_once(android_logger::Config::default().with_max_level(log::LevelFilter::Info).with_tag("VectorCraft"));
    let data_dir = android.internal_data_path();
    let mut options = eframe::NativeOptions { android_app: Some(android), ..Default::default() };
    // The canvas is rasterized on the CPU and the GPU only composites it: one frame queued.
    options.wgpu_options.surface = eframe::egui_wgpu::SurfaceConfig::LOW_LATENCY;
    let result = eframe::run_native(
        "VectorCraft",
        options,
        Box::new(move |_cc| {
            let mut app = VectorcraftApp::new(Session::new(), services());
            if let Some(dir) = data_dir {
                let recovery = dir.join("Data Recovery").to_string_lossy().to_string();
                app.session.recovery.set_default_folder(Some(recovery));
            }
            Ok(Box::new(App { app }))
        }),
    );
    if let Err(e) = result {
        log::error!("VectorCraft stopped: {e}");
    }
}
