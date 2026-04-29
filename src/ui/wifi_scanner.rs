// ─────────────────────────────────────────────────────────────
// NetAnalyzer - WiFi Scanner View
// ─────────────────────────────────────────────────────────────

use eframe::egui::{self, RichText, Layout, Align};
use crate::app::AppState;
use crate::theme;
use crate::wifi::WifiNetwork;
use crate::ui::widgets::{progress_bar, status_badge};

#[derive(PartialEq)]
pub enum BandFilter {
    All,
    Band2_4,
    Band5,
}

#[derive(PartialEq)]
pub enum SortBy {
    Signal,
    Ssid,
    Channel,
}

pub struct WifiScannerState {
    pub networks: Vec<WifiNetwork>,
    pub is_scanning: bool,
    pub error: Option<String>,
    pub band_filter: BandFilter,
    pub sort_by: SortBy,
    pub search_query: String,
}

impl Default for WifiScannerState {
    fn default() -> Self {
        Self {
            networks: Vec::new(),
            is_scanning: false,
            error: None,
            band_filter: BandFilter::All,
            sort_by: SortBy::Signal,
            search_query: String::new(),
        }
    }
}

pub fn show_wifi_scanner(
    ui: &mut egui::Ui,
    state: &mut WifiScannerState,
    app_state: &mut AppState,
) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        // Header
        ui.label(RichText::new("WiFi Scanner").size(24.0).color(theme::TEXT_PRIMARY).strong());
        ui.label(RichText::new("Discover nearby WiFi networks").size(12.0).color(theme::TEXT_SECONDARY));
        ui.add_space(16.0);

        // Controls
        theme::card_frame().show(ui, |ui| {
            ui.horizontal(|ui| {
                if ui.add(
                    egui::Button::new(
                        RichText::new(if state.is_scanning { "⏳ Scanning..." } else { "📡  Scan Networks" })
                            .size(13.0)
                            .color(theme::TEXT_ON_PRIMARY),
                    )
                    .fill(if state.is_scanning { theme::BG_ELEVATED } else { theme::SUCCESS })
                    .corner_radius(egui::CornerRadius::same(6))
                    .min_size(egui::Vec2::new(150.0, 32.0)),
                ).clicked() && !state.is_scanning {
                    perform_wifi_scan(state, app_state);
                }

                ui.add_space(16.0);

                // Band filter
                ui.label(RichText::new("Band:").size(12.0).color(theme::TEXT_SECONDARY));
                if ui.selectable_label(state.band_filter == BandFilter::All, "All").clicked() {
                    state.band_filter = BandFilter::All;
                }
                if ui.selectable_label(state.band_filter == BandFilter::Band2_4, "2.4 GHz").clicked() {
                    state.band_filter = BandFilter::Band2_4;
                }
                if ui.selectable_label(state.band_filter == BandFilter::Band5, "5 GHz").clicked() {
                    state.band_filter = BandFilter::Band5;
                }

                ui.add_space(16.0);

                // Sort
                ui.label(RichText::new("Sort:").size(12.0).color(theme::TEXT_SECONDARY));
                if ui.selectable_label(state.sort_by == SortBy::Signal, "Signal").clicked() {
                    state.sort_by = SortBy::Signal;
                }
                if ui.selectable_label(state.sort_by == SortBy::Ssid, "SSID").clicked() {
                    state.sort_by = SortBy::Ssid;
                }
                if ui.selectable_label(state.sort_by == SortBy::Channel, "Channel").clicked() {
                    state.sort_by = SortBy::Channel;
                }

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if !state.networks.is_empty() {
                        if ui.add(
                            egui::Button::new(RichText::new("💾  Export").size(12.0).color(theme::TEXT_SECONDARY))
                                .fill(theme::BG_ELEVATED)
                                .corner_radius(egui::CornerRadius::same(6)),
                        ).clicked() {
                            export_results(&state.networks);
                        }
                    }
                    
                    ui.label(
                        RichText::new(format!("{} networks found", state.networks.len()))
                            .size(12.0)
                            .color(theme::TEXT_SECONDARY),
                    );
                });
            });

            // Search
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new("🔎").size(14.0));
                let search = egui::TextEdit::singleline(&mut state.search_query)
                    .desired_width(250.0)
                    .hint_text("Search by SSID...");
                ui.add(search);
            });

            if let Some(ref error) = state.error {
                ui.add_space(8.0);
                ui.colored_label(theme::ERROR, format!("⚠ {}", error));
            }
        });

        ui.add_space(8.0);

        // Network list
        if !state.networks.is_empty() {
            let mut filtered: Vec<&WifiNetwork> = state.networks.iter()
                .filter(|n| {
                    let band_ok = match state.band_filter {
                        BandFilter::All => true,
                        BandFilter::Band2_4 => n.band.contains("2.4"),
                        BandFilter::Band5 => n.band.contains("5"),
                    };
                    let search_ok = state.search_query.is_empty()
                        || n.ssid.to_lowercase().contains(&state.search_query.to_lowercase());
                    band_ok && search_ok
                })
                .collect();

            // Sort
            match state.sort_by {
                SortBy::Signal => filtered.sort_by(|a, b| b.signal_percent.cmp(&a.signal_percent)),
                SortBy::Ssid => filtered.sort_by(|a, b| a.ssid.to_lowercase().cmp(&b.ssid.to_lowercase())),
                SortBy::Channel => filtered.sort_by(|a, b| a.channel.cmp(&b.channel)),
            }

            // Render as table
            theme::card_frame().show(ui, |ui| {
                egui_extras::TableBuilder::new(ui)
                    .striped(true)
                    .resizable(true)
                    .cell_layout(Layout::left_to_right(Align::Center))
                    .column(egui_extras::Column::remainder().at_least(120.0)) // SSID
                    .column(egui_extras::Column::exact(60.0))   // Signal
                    .column(egui_extras::Column::exact(50.0))   // Ch
                    .column(egui_extras::Column::exact(70.0))   // Band
                    .column(egui_extras::Column::exact(80.0))   // Security
                    .column(egui_extras::Column::exact(140.0))  // BSSID
                    .header(26.0, |mut header| {
                        header.col(|ui| { ui.label(RichText::new("SSID").size(11.0).color(theme::TEXT_MUTED).strong()); });
                        header.col(|ui| { ui.label(RichText::new("Signal").size(11.0).color(theme::TEXT_MUTED).strong()); });
                        header.col(|ui| { ui.label(RichText::new("Ch").size(11.0).color(theme::TEXT_MUTED).strong()); });
                        header.col(|ui| { ui.label(RichText::new("Band").size(11.0).color(theme::TEXT_MUTED).strong()); });
                        header.col(|ui| { ui.label(RichText::new("Security").size(11.0).color(theme::TEXT_MUTED).strong()); });
                        header.col(|ui| { ui.label(RichText::new("BSSID").size(11.0).color(theme::TEXT_MUTED).strong()); });
                    })
                    .body(|body| {
                        body.rows(28.0, filtered.len(), |mut row| {
                            let net = filtered[row.index()];

                            row.col(|ui| {
                                let ssid = if net.ssid.is_empty() { "(Hidden)" } else { &net.ssid };
                                ui.label(RichText::new(ssid).size(12.0).color(theme::TEXT_PRIMARY).strong());
                            });
                            row.col(|ui| {
                                progress_bar::signal_bar(ui, net.signal_percent, 40.0);
                            });
                            row.col(|ui| {
                                ui.label(RichText::new(format!("{}", net.channel)).size(12.0).color(theme::TEXT_PRIMARY));
                            });
                            row.col(|ui| {
                                ui.label(RichText::new(&net.band).size(11.0).color(theme::TEXT_SECONDARY));
                            });
                            row.col(|ui| {
                                status_badge::security_badge(ui, &net.security);
                            });
                            row.col(|ui| {
                                ui.label(RichText::new(&net.bssid).size(10.0).color(theme::TEXT_MUTED));
                            });
                        });
                    });
            });
        } else if !state.is_scanning {
            theme::card_frame().show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(30.0);
                    ui.label(RichText::new("📡").size(40.0));
                    ui.add_space(8.0);
                    ui.label(RichText::new("No networks scanned yet").size(14.0).color(theme::TEXT_SECONDARY));
                    ui.label(RichText::new("Click 'Scan Networks' to discover nearby WiFi networks").size(12.0).color(theme::TEXT_MUTED));
                    ui.add_space(30.0);
                });
            });
        }
    });
}

fn perform_wifi_scan(state: &mut WifiScannerState, app_state: &mut AppState) {
    state.is_scanning = true;
    state.error = None;

    match crate::wifi::scan_wifi() {
        Ok(networks) => {
            app_state.last_wifi_count = networks.len();
            state.networks = networks;
        }
        Err(e) => {
            state.error = Some(e);
        }
    }

    state.is_scanning = false;
}

fn export_results(results: &[WifiNetwork]) {
    if let Some(path) = rfd_save_dialog("wifi_scan_results.json") {
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
