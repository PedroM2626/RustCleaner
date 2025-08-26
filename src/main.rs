use env_logger;
use log::info;

mod app;
mod scanner;
mod duplicate_finder;
mod cleaner;
mod config;
mod file_category;
mod progress;

use app::DiskCleanerApp;

fn main() -> Result<(), eframe::Error> {
    env_logger::init();
    info!("Starting Intelligent Disk Cleaner");

    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(1200.0, 800.0)),
        min_window_size: Some(egui::vec2(800.0, 600.0)),
        ..Default::default()
    };

    eframe::run_native(
        "Intelligent Disk Cleaner",
        options,
        Box::new(|_cc| Box::new(DiskCleanerApp::new())),
    )
}
