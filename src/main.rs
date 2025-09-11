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
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Intelligent Disk Cleaner",
        options,
        Box::new(|_cc| Ok(Box::new(DiskCleanerApp::new()))),
    )
}
