// ─────────────────────────────────────────────────────────────
// NetAnalyzer - Speed Test View
// ─────────────────────────────────────────────────────────────

use eframe::egui::{self, RichText, Layout, Align};
use crate::app::AppState;
use crate::theme;
use crate::speed::http_test::{SpeedTestPhase, SpeedTestResult};
use crate::ui::widgets::gauge::SpeedGauge;
use tokio::sync::mpsc;

pub struct SpeedTestState {
    pub phase: SpeedTestPhase,
    pub current_speed: f64,
    pub max_gauge: f64,
    pub results: Vec<SpeedTestResult>,
    pub is_testing: bool,
}

impl Default for SpeedTestState {
    fn default() -> Self {
        Self {
            phase: SpeedTestPhase::Idle,
            current_speed: 0.0,
            max_gauge: 100.0,
            results: Vec::new(),
            is_testing: false,
        }
    }
}

pub fn show_speed_test(
    ui: &mut egui::Ui,
    state: &mut SpeedTestState,
    app_state: &mut AppState,
    runtime: &tokio::runtime::Runtime,
    rx: &mut Option<mpsc::UnboundedReceiver<SpeedTestPhase>>,
) {
    // Process incoming updates
    if let Some(receiver) = rx.as_mut() {
        while let Ok(phase) = receiver.try_recv() {
            match &phase {
                SpeedTestPhase::TestingDownload { speed_mbps, .. } => {
                    state.current_speed = *speed_mbps;
                    // Auto-scale gauge
                    if *speed_mbps > state.max_gauge * 0.8 {
                        state.max_gauge = (*speed_mbps * 1.5).max(100.0);
                    }
                }
                SpeedTestPhase::TestingUpload { speed_mbps, .. } => {
                    state.current_speed = *speed_mbps;
                }
                SpeedTestPhase::Complete(result) => {
                    state.is_testing = false;
                    app_state.last_speed_result = format!("↓{:.1} ↑{:.1} Mbps", result.download_mbps, result.upload_mbps);
                    state.results.push(result.clone());
                }
                SpeedTestPhase::Error(_) => {
                    state.is_testing = false;
                }
                _ => {}
            }
            state.phase = phase;
        }
    }

    egui::ScrollArea::vertical().show(ui, |ui| {
        // Header
        ui.label(RichText::new("Speed Test").size(24.0).color(theme::TEXT_PRIMARY).strong());
        ui.label(RichText::new("Test your internet connection speed").size(12.0).color(theme::TEXT_SECONDARY));
        ui.add_space(16.0);

        // Main gauge area
        theme::card_frame().show(ui, |ui| {
            ui.vertical_centered(|ui| {
                // Phase indicator
                let phase_text = match &state.phase {
                    SpeedTestPhase::Idle => "Ready to test",
                    SpeedTestPhase::TestingPing => "⏳ Testing latency...",
                    SpeedTestPhase::TestingDownload { .. } => "⬇️  Testing download...",
                    SpeedTestPhase::TestingUpload { .. } => "⬆️  Testing upload...",
                    SpeedTestPhase::Complete(_) => "✅ Test complete!",
                    SpeedTestPhase::Error(e) => "❌ Error",
                };

                ui.label(RichText::new(phase_text).size(14.0).color(theme::TEXT_SECONDARY));
                ui.add_space(8.0);

                // Speed gauge
                let gauge_label = match &state.phase {
                    SpeedTestPhase::TestingDownload { .. } => "Download",
                    SpeedTestPhase::TestingUpload { .. } => "Upload",
                    SpeedTestPhase::Complete(r) => "Download",
                    _ => "Speed",
                };

                let gauge_value = match &state.phase {
                    SpeedTestPhase::Complete(r) => r.download_mbps,
                    _ => state.current_speed,
                };

                let gauge = SpeedGauge::new(gauge_value, state.max_gauge, gauge_label);
                gauge.show(ui, 250.0);

                ui.add_space(16.0);

                // Start button
                if !state.is_testing {
                    if ui.add(
                        egui::Button::new(
                            RichText::new("▶  Start Speed Test")
                                .size(15.0)
                                .color(theme::TEXT_ON_PRIMARY),
                        )
                        .fill(theme::ACCENT)
                        .corner_radius(egui::CornerRadius::same(8))
                        .min_size(egui::Vec2::new(200.0, 40.0)),
                    ).clicked() {
                        start_speed_test(state, runtime, rx);
                    }
                } else {
                    ui.spinner();
                }
            });
        });

        ui.add_space(8.0);

        // Results cards
        if let SpeedTestPhase::Complete(result) = &state.phase {
            ui.columns(4, |cols| {
                result_card(&mut cols[0], "⬇️  Download", &format!("{:.1} Mbps", result.download_mbps), theme::PRIMARY);
                result_card(&mut cols[1], "⬆️  Upload", &format!("{:.1} Mbps", result.upload_mbps), theme::SECONDARY);
                result_card(&mut cols[2], "📡  Ping", &format!("{:.1} ms", result.ping_ms), theme::SUCCESS);
                result_card(&mut cols[3], "📊  Jitter", &format!("{:.1} ms", result.jitter_ms), theme::WARNING);
            });
        }

        // Error
        if let SpeedTestPhase::Error(e) = &state.phase {
            ui.add_space(8.0);
            theme::card_frame().show(ui, |ui| {
                ui.colored_label(theme::ERROR, format!("⚠ Error: {}", e));
            });
        }

        // History
        if !state.results.is_empty() {
            ui.add_space(16.0);
            theme::card_frame().show(ui, |ui| {
                ui.label(RichText::new("📜  History").size(14.0).color(theme::TEXT_PRIMARY).strong());
                ui.add_space(8.0);

                egui_extras::TableBuilder::new(ui)
                    .striped(true)
                    .resizable(true)
                    .cell_layout(Layout::left_to_right(Align::Center))
                    .column(egui_extras::Column::exact(160.0))  // Time
                    .column(egui_extras::Column::exact(100.0))  // Download
                    .column(egui_extras::Column::exact(100.0))  // Upload
                    .column(egui_extras::Column::exact(80.0))   // Ping
                    .column(egui_extras::Column::remainder())    // Server
                    .header(24.0, |mut header| {
                        header.col(|ui| { ui.label(RichText::new("Time").size(11.0).color(theme::TEXT_MUTED).strong()); });
                        header.col(|ui| { ui.label(RichText::new("Download").size(11.0).color(theme::TEXT_MUTED).strong()); });
                        header.col(|ui| { ui.label(RichText::new("Upload").size(11.0).color(theme::TEXT_MUTED).strong()); });
                        header.col(|ui| { ui.label(RichText::new("Ping").size(11.0).color(theme::TEXT_MUTED).strong()); });
                        header.col(|ui| { ui.label(RichText::new("Server").size(11.0).color(theme::TEXT_MUTED).strong()); });
                    })
                    .body(|body| {
                        body.rows(22.0, state.results.len(), |mut row| {
                            let idx = state.results.len() - 1 - row.index(); // Newest first
                            let r = &state.results[idx];
                            row.col(|ui| { ui.label(RichText::new(&r.timestamp).size(11.0).color(theme::TEXT_SECONDARY)); });
                            row.col(|ui| { ui.label(RichText::new(format!("{:.1} Mbps", r.download_mbps)).size(12.0).color(theme::PRIMARY)); });
                            row.col(|ui| { ui.label(RichText::new(format!("{:.1} Mbps", r.upload_mbps)).size(12.0).color(theme::SECONDARY)); });
                            row.col(|ui| { ui.label(RichText::new(format!("{:.1} ms", r.ping_ms)).size(12.0).color(theme::SUCCESS)); });
                            row.col(|ui| { ui.label(RichText::new(&r.server).size(11.0).color(theme::TEXT_SECONDARY)); });
                        });
                    });
            });
        }
    });

    if state.is_testing {
        ui.ctx().request_repaint();
    }
}

fn result_card(ui: &mut egui::Ui, label: &str, value: &str, color: egui::Color32) {
    theme::card_frame().show(ui, |ui| {
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(label).size(11.0).color(theme::TEXT_MUTED));
            ui.add_space(4.0);
            ui.label(RichText::new(value).size(18.0).color(color).strong());
        });
    });
}

fn start_speed_test(
    state: &mut SpeedTestState,
    runtime: &tokio::runtime::Runtime,
    rx: &mut Option<mpsc::UnboundedReceiver<SpeedTestPhase>>,
) {
    state.is_testing = true;
    state.current_speed = 0.0;
    state.phase = SpeedTestPhase::Idle;

    let (tx, rx_chan) = mpsc::unbounded_channel();
    *rx = Some(rx_chan);

    runtime.spawn(async move {
        crate::speed::http_test::run_speed_test(tx).await;
    });
}
