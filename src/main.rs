// ─────────────────────────────────────────────────────────────
// NetAnalyzer - Cross-Platform Network Analysis Tool
// ─────────────────────────────────────────────────────────────

mod app;
mod theme;
mod ui;
mod scanner;
mod speed;
mod wifi;
mod utils;

use eframe::egui;

fn main() -> eframe::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("⚡ NetAnalyzer")
            .with_inner_size([1200.0, 750.0])
            .with_min_inner_size([900.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "NetAnalyzer",
        options,
        Box::new(|cc| Ok(Box::new(app::NetAnalyzerApp::new(cc)))),
    )
}
