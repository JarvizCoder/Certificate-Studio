#![allow(dead_code)]

mod app;
mod data;
mod editor;
mod export;
mod fonts;
mod model;
mod render;

use app::CertificateStudioApp;

fn main() -> eframe::Result {
    tracing_subscriber::fmt::init();

    let native_options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 820.0])
            .with_min_inner_size([900.0, 600.0])
            .with_title("Certificate Studio"),
        ..Default::default()
    };

    eframe::run_native(
        "Certificate Studio",
        native_options,
        Box::new(|cc| Ok(Box::new(CertificateStudioApp::new(cc)))),
    )
}
