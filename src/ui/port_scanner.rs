// ─────────────────────────────────────────────────────────────
// NetAnalyzer - Port Scanner View
// ─────────────────────────────────────────────────────────────

use eframe::egui::{self, RichText, Layout, Align};
use crate::app::AppState;
use crate::theme;
use crate::scanner::port::{PortScanResult, PortScanProgress, PortState};
use crate::ui::widgets::progress_bar::AnimatedProgressBar;
use crate::ui::widgets::status_badge::{self, BadgeType};
use crate::utils::network;
use std::net::Ipv4Addr;
use std::time::Duration;
use tokio::sync::mpsc;

#[derive(PartialEq)]
pub enum PortPreset {
    Top100,
    Top1000,
    Common,
    Custom,
}

pub struct PortScannerState {
    pub target_input: String,
    pub port_input: String,
    pub preset: PortPreset,
    pub timeout_ms: u32,
    pub workers: u32,
    pub grab_banners: bool,
    pub is_scanning: bool,
    pub progress: f32,
    pub scanned: usize,
    pub total: usize,
    pub results: Vec<PortScanResult>,
    pub error: Option<String>,
}

impl Default for PortScannerState {
    fn default() -> Self {
        Self {
            target_input: String::new(),
            port_input: "1-1024".to_string(),
            preset: PortPreset::Common,
            timeout_ms: 1000,
            workers: 200,
            grab_banners: true,
            is_scanning: false,
            progress: 0.0,
            scanned: 0,
            total: 0,
            results: Vec::new(),
            error: None,
        }
    }
}

pub fn show_port_scanner(
    ui: &mut egui::Ui,
    state: &mut PortScannerState,
    app_state: &mut AppState,
    runtime: &tokio::runtime::Runtime,
    rx: &mut Option<mpsc::UnboundedReceiver<PortScanProgress>>,
) {
    // Process incoming results
    if let Some(receiver) = rx.as_mut() {
        while let Ok(msg) = receiver.try_recv() {
            match msg {
                PortScanProgress::Update { scanned, total } => {
                    state.scanned = scanned;
                    state.total = total;
                    state.progress = scanned as f32 / total as f32;
                }
                PortScanProgress::Found(result) => {
                    state.results.push(result);
                    app_state.last_port_count = state.results.len();
                }
                PortScanProgress::Done => {
                    state.is_scanning = false;
                    state.progress = 1.0;
                    // Sort results by port number
                    state.results.sort_by_key(|r| r.port);
                }
                PortScanProgress::Error(e) => {
                    state.error = Some(e);
                    state.is_scanning = false;
                }
            }
        }
    }

    egui::ScrollArea::vertical().show(ui, |ui| {
        // Header
        ui.label(RichText::new("Port Scanner").size(24.0).color(theme::TEXT_PRIMARY).strong());
        ui.label(RichText::new("Scan for open ports on a target host").size(12.0).color(theme::TEXT_SECONDARY));
        ui.add_space(16.0);

        // Controls
        theme::card_frame().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Target:").size(12.0).color(theme::TEXT_SECONDARY));
                let input = egui::TextEdit::singleline(&mut state.target_input)
                    .desired_width(200.0)
                    .hint_text("e.g., 192.168.1.1");
                ui.add(input);

                ui.add_space(16.0);

                ui.label(RichText::new("Timeout (ms):").size(12.0).color(theme::TEXT_SECONDARY));
                ui.add(egui::DragValue::new(&mut state.timeout_ms).range(100..=10000).speed(50));

                ui.label(RichText::new("Workers:").size(12.0).color(theme::TEXT_SECONDARY));
                ui.add(egui::DragValue::new(&mut state.workers).range(1..=1000).speed(10));
            });

            ui.add_space(6.0);

            ui.horizontal(|ui| {
                ui.label(RichText::new("Ports:").size(12.0).color(theme::TEXT_SECONDARY));

                if ui.selectable_label(state.preset == PortPreset::Common, "Common (Top 30)").clicked() {
                    state.preset = PortPreset::Common;
                }
                if ui.selectable_label(state.preset == PortPreset::Top100, "Top 100").clicked() {
                    state.preset = PortPreset::Top100;
                }
                if ui.selectable_label(state.preset == PortPreset::Top1000, "1-1024").clicked() {
                    state.preset = PortPreset::Top1000;
                    state.port_input = "1-1024".to_string();
                }
                if ui.selectable_label(state.preset == PortPreset::Custom, "Custom").clicked() {
                    state.preset = PortPreset::Custom;
                }

                if state.preset == PortPreset::Custom {
                    let input = egui::TextEdit::singleline(&mut state.port_input)
                        .desired_width(150.0)
                        .hint_text("e.g., 22,80,443 or 1-1024");
                    ui.add(input);
                }

                ui.add_space(8.0);
                ui.checkbox(&mut state.grab_banners, "Grab Banners");
            });

            ui.add_space(8.0);

            ui.horizontal(|ui| {
                let can_scan = !state.is_scanning && !state.target_input.is_empty();

                if ui.add_enabled(
                    can_scan,
                    egui::Button::new(
                        RichText::new(if state.is_scanning { "⏳ Scanning..." } else { "▶  Start Scan" })
                            .size(13.0)
                            .color(theme::TEXT_ON_PRIMARY),
                    )
                    .fill(if can_scan { theme::SECONDARY } else { theme::BG_ELEVATED })
                    .corner_radius(egui::CornerRadius::same(6))
                    .min_size(egui::Vec2::new(130.0, 32.0)),
                ).clicked() {
                    start_port_scan(state, runtime, rx);
                }

                ui.add_space(8.0);

                if !state.results.is_empty() {
                    if ui.add(
                        egui::Button::new(RichText::new("🗑  Clear").size(12.0).color(theme::TEXT_SECONDARY))
                            .fill(theme::BG_ELEVATED)
                            .corner_radius(egui::CornerRadius::same(6)),
                    ).clicked() {
                        state.results.clear();
                        state.progress = 0.0;
                        state.error = None;
                    }

                    if ui.add(
                        egui::Button::new(RichText::new("💾  Export JSON").size(12.0).color(theme::TEXT_SECONDARY))
                            .fill(theme::BG_ELEVATED)
                            .corner_radius(egui::CornerRadius::same(6)),
                    ).clicked() {
                        export_results(&state.results);
                    }
                }

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if state.is_scanning {
                        ui.label(
                            RichText::new(format!("{} / {} ports", state.scanned, state.total))
                                .size(12.0)
                                .color(theme::TEXT_SECONDARY),
                        );
                    }
                    ui.label(
                        RichText::new(format!("{} open ports", state.results.len()))
                            .size(12.0)
                            .color(theme::SUCCESS),
                    );
                });
            });

            // Progress
            if state.is_scanning || state.progress > 0.0 {
                ui.add_space(8.0);
                AnimatedProgressBar::new(state.progress, "Scanning ports")
                    .color(if state.is_scanning { theme::SECONDARY } else { theme::SUCCESS })
                    .show(ui);
            }

            if let Some(ref error) = state.error {
                ui.add_space(8.0);
                ui.colored_label(theme::ERROR, format!("⚠ {}", error));
            }
        });

        ui.add_space(8.0);

        // Results table
        if !state.results.is_empty() {
            theme::card_frame().show(ui, |ui| {
                ui.label(RichText::new("Open Ports").size(14.0).color(theme::TEXT_PRIMARY).strong());
                ui.add_space(8.0);

                egui_extras::TableBuilder::new(ui)
                    .striped(true)
                    .resizable(true)
                    .cell_layout(Layout::left_to_right(Align::Center))
                    .column(egui_extras::Column::exact(70.0))   // Port
                    .column(egui_extras::Column::exact(70.0))   // State
                    .column(egui_extras::Column::exact(120.0))  // Service
                    .column(egui_extras::Column::remainder())    // Banner
                    .header(24.0, |mut header| {
                        header.col(|ui| { ui.label(RichText::new("Port").size(11.0).color(theme::TEXT_MUTED).strong()); });
                        header.col(|ui| { ui.label(RichText::new("State").size(11.0).color(theme::TEXT_MUTED).strong()); });
                        header.col(|ui| { ui.label(RichText::new("Service").size(11.0).color(theme::TEXT_MUTED).strong()); });
                        header.col(|ui| { ui.label(RichText::new("Banner").size(11.0).color(theme::TEXT_MUTED).strong()); });
                    })
                    .body(|body| {
                        body.rows(22.0, state.results.len(), |mut row| {
                            let result = &state.results[row.index()];
                            row.col(|ui| {
                                ui.label(RichText::new(format!("{}", result.port)).size(12.0).color(theme::TEXT_PRIMARY).strong());
                            });
                            row.col(|ui| {
                                let (badge_text, badge_type) = match result.state {
                                    PortState::Open => ("Open", BadgeType::Success),
                                    PortState::Closed => ("Closed", BadgeType::Error),
                                    PortState::Filtered => ("Filtered", BadgeType::Warning),
                                };
                                status_badge::status_badge(ui, badge_text, badge_type);
                            });
                            row.col(|ui| {
                                ui.label(RichText::new(&result.service).size(12.0).color(theme::SECONDARY));
                            });
                            row.col(|ui| {
                                let banner = if result.banner.is_empty() { "—".to_string() } else { result.banner.clone() };
                                ui.label(RichText::new(banner).size(11.0).color(theme::TEXT_SECONDARY));
                            });
                        });
                    });
            });
        }
    });

    if state.is_scanning {
        ui.ctx().request_repaint();
    }
}

fn start_port_scan(
    state: &mut PortScannerState,
    runtime: &tokio::runtime::Runtime,
    rx: &mut Option<mpsc::UnboundedReceiver<PortScanProgress>>,
) {
    let target: Ipv4Addr = match state.target_input.trim().parse() {
        Ok(ip) => ip,
        Err(_) => {
            // Try DNS resolution
            match dns_lookup::lookup_host(state.target_input.trim()) {
                Ok(ips) => {
                    if let Some(std::net::IpAddr::V4(ip)) = ips.into_iter().find(|ip| ip.is_ipv4()) {
                        ip
                    } else {
                        state.error = Some("Could not resolve hostname to IPv4".to_string());
                        return;
                    }
                }
                Err(e) => {
                    state.error = Some(format!("Invalid target: {}", e));
                    return;
                }
            }
        }
    };

    let ports = match state.preset {
        PortPreset::Common => network::top_ports(30),
        PortPreset::Top100 => network::top_ports(100),
        PortPreset::Top1000 => {
            match network::parse_ports(&state.port_input) {
                Ok(p) => p,
                Err(e) => { state.error = Some(e); return; }
            }
        }
        PortPreset::Custom => {
            match network::parse_ports(&state.port_input) {
                Ok(p) => p,
                Err(e) => { state.error = Some(e); return; }
            }
        }
    };

    state.is_scanning = true;
    state.results.clear();
    state.progress = 0.0;
    state.total = ports.len();
    state.error = None;

    let (tx_chan, rx_chan) = mpsc::unbounded_channel();
    *rx = Some(rx_chan);

    let timeout = Duration::from_millis(state.timeout_ms as u64);
    let workers = state.workers as usize;
    let grab_banners = state.grab_banners;

    runtime.spawn(async move {
        crate::scanner::port::scan_ports(target, ports, timeout, workers, grab_banners, tx_chan).await;
    });
}

fn export_results(results: &[PortScanResult]) {
    if let Some(path) = rfd_save_dialog("port_scan_results.json") {
        match serde_json::to_string_pretty(results) {
            Ok(json) => {
                let _ = std::fs::write(&path, json);
            }
            Err(e) => {
                tracing::error!("Failed to serialize results: {}", e);
            }
        }
    }
}

fn rfd_save_dialog(default_name: &str) -> Option<std::path::PathBuf> {
    rfd::FileDialog::new()
        .set_file_name(default_name)
        .add_filter("JSON", &["json"])
        .save_file()
}
