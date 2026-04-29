// ─────────────────────────────────────────────────────────────
// NetAnalyzer - Main Application
// ─────────────────────────────────────────────────────────────

use eframe::egui;
use tokio::sync::mpsc;
use crate::theme;
use crate::ui;
use crate::scanner::ip::ScanProgress;
use crate::scanner::port::PortScanProgress;
use crate::speed::http_test::SpeedTestPhase;

// ── Page Navigation ────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum Page {
    Dashboard,
    IpScanner,
    PortScanner,
    SpeedTest,
    WifiScanner,
    ChannelAnalyzer,
}

// ── Shared Application State ───────────────────────────────

pub struct AppState {
    pub local_ip: String,
    pub hostname: String,
    pub os_info: String,
    pub last_ip_count: usize,
    pub last_port_count: usize,
    pub last_speed_result: String,
    pub last_wifi_count: usize,
}

impl Default for AppState {
    fn default() -> Self {
        let local_ip = local_ip_address::local_ip()
            .map(|ip| ip.to_string())
            .unwrap_or_else(|_| "Unknown".to_string());

        let hostname = hostname::get()
            .map(|h| h.to_string_lossy().to_string())
            .unwrap_or_else(|_| "Unknown".to_string());

        let os_info = format!("{} {}", std::env::consts::OS, std::env::consts::ARCH);

        Self {
            local_ip,
            hostname,
            os_info,
            last_ip_count: 0,
            last_port_count: 0,
            last_speed_result: "Not tested".to_string(),
            last_wifi_count: 0,
        }
    }
}

// ── Main Application ───────────────────────────────────────

pub struct NetAnalyzerApp {
    current_page: Page,
    app_state: AppState,

    // Tokio runtime for async operations
    runtime: tokio::runtime::Runtime,

    // Page states
    ip_scanner_state: ui::ip_scanner::IpScannerState,
    port_scanner_state: ui::port_scanner::PortScannerState,
    speed_test_state: ui::speed_test::SpeedTestState,
    wifi_scanner_state: ui::wifi_scanner::WifiScannerState,
    channel_analyzer_state: ui::channel_analyzer::ChannelAnalyzerState,

    // Async channels
    ip_scan_rx: Option<mpsc::UnboundedReceiver<ScanProgress>>,
    port_scan_rx: Option<mpsc::UnboundedReceiver<PortScanProgress>>,
    speed_test_rx: Option<mpsc::UnboundedReceiver<SpeedTestPhase>>,

    // Theme applied flag
    theme_applied: bool,
}

impl NetAnalyzerApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .worker_threads(4)
            .build()
            .expect("Failed to create Tokio runtime");

        Self {
            current_page: Page::Dashboard,
            app_state: AppState::default(),
            runtime,
            ip_scanner_state: ui::ip_scanner::IpScannerState::new(),
            port_scanner_state: ui::port_scanner::PortScannerState::default(),
            speed_test_state: ui::speed_test::SpeedTestState::default(),
            wifi_scanner_state: ui::wifi_scanner::WifiScannerState::default(),
            channel_analyzer_state: ui::channel_analyzer::ChannelAnalyzerState::default(),
            ip_scan_rx: None,
            port_scan_rx: None,
            speed_test_rx: None,
            theme_applied: false,
        }
    }
}

impl eframe::App for NetAnalyzerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Apply theme once
        if !self.theme_applied {
            theme::apply_theme(ctx);
            self.theme_applied = true;
        }

        // Sidebar panel
        egui::SidePanel::left("sidebar")
            .resizable(false)
            .exact_width(190.0)
            .show(ctx, |ui| {
                ui::sidebar::show_sidebar(ui, &mut self.current_page);
            });

        // Status bar at bottom
        egui::TopBottomPanel::bottom("statusbar")
            .exact_height(24.0)
            .show(ctx, |ui| {
                theme::status_frame().show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(format!("🌐 {}", self.app_state.local_ip))
                                .size(10.0)
                                .color(theme::TEXT_MUTED),
                        );
                        ui.separator();
                        ui.label(
                            egui::RichText::new(format!("🖥 {}", self.app_state.hostname))
                                .size(10.0)
                                .color(theme::TEXT_MUTED),
                        );

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(
                                egui::RichText::new(&self.app_state.os_info)
                                    .size(10.0)
                                    .color(theme::TEXT_MUTED),
                            );
                            ui.separator();
                            // Show scan status if active
                            let status = if self.ip_scanner_state.is_scanning {
                                "IP Scan running..."
                            } else if self.port_scanner_state.is_scanning {
                                "Port Scan running..."
                            } else if self.speed_test_state.is_testing {
                                "Speed Test running..."
                            } else {
                                "Ready"
                            };
                            let color = if status == "Ready" { theme::SUCCESS } else { theme::WARNING };
                            ui.label(
                                egui::RichText::new(status)
                                    .size(10.0)
                                    .color(color),
                            );
                        });
                    });
                });
            });

        // Main content area
        egui::CentralPanel::default().show(ctx, |ui| {
            theme::content_frame().show(ui, |ui| {
                match self.current_page {
                    Page::Dashboard => {
                        ui::dashboard::show_dashboard(
                            ui,
                            &self.app_state,
                            &mut self.current_page,
                        );
                    }
                    Page::IpScanner => {
                        ui::ip_scanner::show_ip_scanner(
                            ui,
                            &mut self.ip_scanner_state,
                            &mut self.app_state,
                            &self.runtime,
                            &mut self.ip_scan_rx,
                        );
                    }
                    Page::PortScanner => {
                        ui::port_scanner::show_port_scanner(
                            ui,
                            &mut self.port_scanner_state,
                            &mut self.app_state,
                            &self.runtime,
                            &mut self.port_scan_rx,
                        );
                    }
                    Page::SpeedTest => {
                        ui::speed_test::show_speed_test(
                            ui,
                            &mut self.speed_test_state,
                            &mut self.app_state,
                            &self.runtime,
                            &mut self.speed_test_rx,
                        );
                    }
                    Page::WifiScanner => {
                        ui::wifi_scanner::show_wifi_scanner(
                            ui,
                            &mut self.wifi_scanner_state,
                            &mut self.app_state,
                        );
                    }
                    Page::ChannelAnalyzer => {
                        ui::channel_analyzer::show_channel_analyzer(
                            ui,
                            &mut self.channel_analyzer_state,
                            &self.wifi_scanner_state.networks,
                        );
                    }
                }
            });
        });
    }
}
