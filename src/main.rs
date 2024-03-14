#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

pub mod app;
mod shapes_widget;

use dust_dds::configuration::DustDdsConfigurationBuilder;

fn main() -> Result<(), eframe::Error> {
    const ICON: &[u8] = include_bytes!("../res/logo.png");
    let icon = eframe::icon_data::from_png_bytes(ICON).expect("Failed to open icon");
    let viewport = eframe::egui::viewport::ViewportBuilder {
        // min_inner_size: Some(eframe::egui::vec2(500.0, 300.0)),
        icon: Some(std::sync::Arc::new(icon)),
        ..Default::default()
    };
    let options = eframe::NativeOptions {
        viewport,
        default_theme: eframe::Theme::Light,
        ..Default::default()
    };
    let configuration = DustDdsConfigurationBuilder::new()
        .interface_name(Some("Wi-Fi".to_string()))
        .build()
        .unwrap();
    eframe::run_native(
        "Dust DDS Shapes Demo",
        options,
        Box::new(|_cc| Box::new(app::ShapesDemoApp::new(Some(configuration)))),
    )
}
