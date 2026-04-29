// ─────────────────────────────────────────────────────────────
// NetAnalyzer - IP Scanner View
// ─────────────────────────────────────────────────────────────

use eframe::egui::{self, RichText, Layout, Align};
use crate::app::AppState;
use crate::theme;
use crate::scanner::ip::{IpScanResult, ScanProgress};
use crate::ui::widgets::progress_bar::AnimatedProgressBar;
use crate::ui::widgets::status_badge::{self, BadgeType};
use crate::utils::network;
use std::time::Duration;
use tokio::sync::mpsc;

#[derive(Default)]
pub struct IpScannerState {
    pub cidr_input: String,
    pub timeout_ms: u32,
    pub workers: u32,
    pub is_scanning: bool,
    pub progress: f32,
    pub scanned: usize,
    pub total: usize,
    pub results: Vec<IpScanResult>,
    pub error: Option<String>,
    pub sort_by: SortColumn,
    pub sort_ascending: bool,
}

#[derive(Default, PartialEq)]
pub enum SortColumn {
    #[default]
    Ip,
    Hostname,
    Latency,
}

impl IpScannerState {
    pub fn new() -> Self {
        Self {
            cidr_input: "192.168.1.0/24".to_string(),
            timeout_ms: 1000,
            workers: 100,
            ..Default::default()
        }
    }
}

pub fn show_ip_scanner(
    ui: &mut egui::Ui,
    state: &mut IpScannerState,
    app_state: &mut AppState,
    runtime: &tokio::runtime::Runtime,
    rx: &mut Option<mpsc::UnboundedReceiver<ScanProgress>>,
) {
    // Process incoming results
    if let Some(receiver) = rx.as_mut() {
        while let Ok(msg) = receiver.try_recv() {
            match msg {
                ScanProgress::Update { scanned, total } => {
                    state.scanned = scanned;
                    state.total = total;
                    state.progress = scanned as f32 / total as f32;
                }
                ScanProgress::Found(result) => {
                    state.results.push(result);
                    app_state.last_ip_count = state.results.len();
                }
                ScanProgress::Done => {
                    state.is_scanning = false;
                    state.progress = 1.0;
                }
                ScanProgress::Error(e) => {
                    state.error = Some(e);
                    state.is_scanning = false;
                }
            }
        }
    }

    egui::ScrollArea::vertical().show(ui, |ui| {
        // Header
        ui.label(RichText::new("IP Scanner").size(24.0).color(theme::TEXT_PRIMARY).strong());
        ui.label(RichText::new("Discover active hosts on your network").size(12.0).color(theme::TEXT_SECONDARY));
        ui.add_space(16.0);

        // Controls
        theme::card_frame().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Target (CIDR):").size(12.0).color(theme::TEXT_SECONDARY));
                let input = egui::TextEdit::singleline(&mut state.cidr_input)
                    .desired_width(200.0)
                    .hint_text("e.g., 192.168.1.0/24");
                ui.add(input);

                ui.add_space(16.0);

                ui.label(RichText::new("Timeout (ms):").size(12.0).color(theme::TEXT_SECONDARY));
                ui.add(egui::DragValue::new(&mut state.timeout_ms).range(100..=10000).speed(50));

                ui.label(RichText::new("Workers:").size(12.0).color(theme::TEXT_SECONDARY));
                ui.add(egui::DragValue::new(&mut state.workers).range(1..=500).speed(5));
            });

            ui.add_space(8.0);

            ui.horizontal(|ui| {
                let can_scan = !state.is_scanning && !state.cidr_input.is_empty();

                if ui.add_enabled(
                    can_scan,
                    egui::Button::new(
                        RichText::new(if state.is_scanning { "⏳ Scanning..." } else { "▶  Start Scan" })
                            .size(13.0)
                            .color(theme::TEXT_ON_PRIMARY),
                    )
                    .fill(if can_scan { theme::PRIMARY } else { theme::BG_ELEVATED })
                    .corner_radius(egui::CornerRadius::same(6))
                    .min_size(egui::Vec2::new(130.0, 32.0)),
                ).clicked() {
                    start_scan(state, runtime, rx);
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
                        state.scanned = 0;
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
                            RichText::new(format!("{} / {} hosts", state.scanned, state.total))
                                .size(12.0)
                                .color(theme::TEXT_SECONDARY),
                        );
                    }
                    ui.label(
                        RichText::new(format!("{} hosts found", state.results.len()))
                            .size(12.0)
                            .color(theme::SUCCESS),
                    );
                });
            });

            // Progress bar
            if state.is_scanning || state.progress > 0.0 {
                ui.add_space(8.0);
                AnimatedProgressBar::new(state.progress, "Scanning")
                    .color(if state.is_scanning { theme::PRIMARY } else { theme::SUCCESS })
                    .show(ui);
            }

            // Error display
            if let Some(ref error) = state.error {
                ui.add_space(8.0);
                ui.colored_label(theme::ERROR, format!("⚠ {}", error));
            }
        });

        ui.add_space(8.0);

        // Results table
        if !state.results.is_empty() {
            theme::card_frame().show(ui, |ui| {
                ui.label(RichText::new("Results").size(14.0).color(theme::TEXT_PRIMARY).strong());
                ui.add_space(8.0);

                egui_extras::TableBuilder::new(ui)
                    .striped(true)
                    .resizable(true)
                    .cell_layout(Layout::left_to_right(Align::Center))
                    .column(egui_extras::Column::exact(140.0)) // IP
                    .column(egui_extras::Column::remainder())   // Hostname
                    .column(egui_extras::Column::exact(90.0))   // Latency
                    .column(egui_extras::Column::exact(70.0))   // Method
                    .column(egui_extras::Column::exact(70.0))   // Status
                    .header(24.0, |mut header| {
                        header.col(|ui| { ui.label(RichText::new("IP Address").size(11.0).color(theme::TEXT_MUTED).strong()); });
                        header.col(|ui| { ui.label(RichText::new("Hostname").size(11.0).color(theme::TEXT_MUTED).strong()); });
                        header.col(|ui| { ui.label(RichText::new("Latency").size(11.0).color(theme::TEXT_MUTED).strong()); });
                        header.col(|ui| { ui.label(RichText::new("Method").size(11.0).color(theme::TEXT_MUTED).strong()); });
                        header.col(|ui| { ui.label(RichText::new("Status").size(11.0).color(theme::TEXT_MUTED).strong()); });
                    })
                    .body(|body| {
                        body.rows(22.0, state.results.len(), |mut row| {
                            let result = &state.results[row.index()];
                            row.col(|ui| { ui.label(RichText::new(&result.ip).size(12.0).color(theme::TEXT_PRIMARY)); });
                            row.col(|ui| {
                                let host = if result.hostname.is_empty() { "—" } else { &result.hostname };
                                ui.label(RichText::new(host).size(12.0).color(theme::TEXT_SECONDARY));
                            });
                            row.col(|ui| {
                                let latency = crate::utils::network::format_latency(result.latency_ms);
                                let color = if result.latency_ms < 10.0 { theme::SUCCESS }
                                    else if result.latency_ms < 100.0 { theme::WARNING }
                                    else { theme::ERROR };
                                ui.label(RichText::new(latency).size(12.0).color(color));
                            });
                            row.col(|ui| { ui.label(RichText::new(&result.method).size(11.0).color(theme::TEXT_MUTED)); });
                            row.col(|ui| { status_badge::status_badge(ui, "Alive", BadgeType::Success); });
                        });
                    });
            });
        }
    });

    // Keep UI updating during scan
    if state.is_scanning {
        ui.ctx().request_repaint();
    }
}

fn start_scan(
    state: &mut IpScannerState,
    runtime: &tokio::runtime::Runtime,
    rx: &mut Option<mpsc::UnboundedReceiver<ScanProgress>>,
) {
    match network::parse_cidr(&state.cidr_input) {
        Ok(ips) => {
            state.is_scanning = true;
            state.results.clear();
            state.progress = 0.0;
            state.scanned = 0;
            state.total = ips.len();
            state.error = None;

            let (tx_chan, rx_chan) = mpsc::unbounded_channel();
            *rx = Some(rx_chan);

            let timeout = Duration::from_millis(state.timeout_ms as u64);
            let workers = state.workers as usize;

            runtime.spawn(async move {
                crate::scanner::ip::scan_ips(ips, timeout, workers, tx_chan).await;
            });
        }
        Err(e) => {
            state.error = Some(e);
        }
    }
}

fn export_results(results: &[IpScanResult]) {
    if let Some(path) = rfd_save_dialog("ip_scan_results.json") {
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
