// ─────────────────────────────────────────────────────────────
// NetAnalyzer - Channel Analyzer View
// ─────────────────────────────────────────────────────────────

use eframe::egui::{self, RichText, Layout, Align};
use egui_plot::{Bar, BarChart, Plot};
use crate::theme;
use crate::wifi::{WifiNetwork, channel::{ChannelAnalysis, ChannelInfo, CongestionLevel, analyze_channels}};
use crate::ui::widgets::status_badge::{self, BadgeType};

#[derive(PartialEq)]
pub enum ChannelTab {
    Band2_4,
    Band5,
}

pub struct ChannelAnalyzerState {
    pub analysis: Option<ChannelAnalysis>,
    pub tab: ChannelTab,
    pub error: Option<String>,
}

impl Default for ChannelAnalyzerState {
    fn default() -> Self {
        Self {
            analysis: None,
            tab: ChannelTab::Band2_4,
            error: None,
        }
    }
}

pub fn show_channel_analyzer(
    ui: &mut egui::Ui,
    state: &mut ChannelAnalyzerState,
    wifi_networks: &[WifiNetwork],
) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        // Header
        ui.label(RichText::new("Channel Analyzer").size(24.0).color(theme::TEXT_PRIMARY).strong());
        ui.label(RichText::new("Analyze WiFi channel congestion and find the best channel").size(12.0).color(theme::TEXT_SECONDARY));
        ui.add_space(16.0);

        // Controls
        theme::card_frame().show(ui, |ui| {
            ui.horizontal(|ui| {
                if ui.add(
                    egui::Button::new(
                        RichText::new("📊  Analyze Channels")
                            .size(13.0)
                            .color(theme::TEXT_ON_PRIMARY),
                    )
                    .fill(theme::ACCENT)
                    .corner_radius(egui::CornerRadius::same(6))
                    .min_size(egui::Vec2::new(160.0, 32.0)),
                ).clicked() {
                    perform_analysis(state, wifi_networks);
                }

                ui.add_space(16.0);

                // Tab selector
                if ui.selectable_label(state.tab == ChannelTab::Band2_4, "2.4 GHz").clicked() {
                    state.tab = ChannelTab::Band2_4;
                }
                if ui.selectable_label(state.tab == ChannelTab::Band5, "5 GHz").clicked() {
                    state.tab = ChannelTab::Band5;
                }
            });

            if let Some(ref error) = state.error {
                ui.add_space(8.0);
                ui.colored_label(theme::ERROR, format!("⚠ {}", error));
            }
        });

        ui.add_space(8.0);

        // Analysis results
        if let Some(ref analysis) = state.analysis {
            let channels = match state.tab {
                ChannelTab::Band2_4 => &analysis.channels_2g,
                ChannelTab::Band5 => &analysis.channels_5g,
            };

            let recommended = match state.tab {
                ChannelTab::Band2_4 => analysis.recommended_2g,
                ChannelTab::Band5 => analysis.recommended_5g,
            };

            // Recommendation card
            theme::card_frame().show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("✨").size(20.0));
                    ui.vertical(|ui| {
                        ui.label(RichText::new("Recommended Channel").size(14.0).color(theme::TEXT_PRIMARY).strong());
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(format!("Channel {}", recommended))
                                    .size(22.0)
                                    .color(theme::SUCCESS)
                                    .strong(),
                            );
                            ui.label(
                                RichText::new("(least congested)")
                                    .size(12.0)
                                    .color(theme::TEXT_SECONDARY),
                            );
                        });
                    });
                });
            });

            ui.add_space(8.0);

            // Bar chart
            theme::card_frame().show(ui, |ui| {
                ui.label(RichText::new("Networks per Channel").size(14.0).color(theme::TEXT_PRIMARY).strong());
                ui.add_space(8.0);

                let bars: Vec<Bar> = channels
                    .iter()
                    .filter(|ch| ch.channel > 0)
                    .map(|ch| {
                        let color = match ch.congestion {
                            CongestionLevel::Low => theme::SUCCESS,
                            CongestionLevel::Medium => theme::WARNING,
                            CongestionLevel::High => theme::ERROR,
                        };

                        Bar::new(ch.channel as f64, ch.network_count as f64)
                            .fill(color)
                            .width(0.8)
                            .name(format!("Ch {}", ch.channel))
                    })
                    .collect();

                let chart = BarChart::new(bars).name("Networks");

                let plot = Plot::new("channel_plot")
                    .height(250.0)
                    .allow_drag(false)
                    .allow_zoom(false)
                    .allow_scroll(false)
                    .show_axes([true, true])
                    .x_axis_label("Channel")
                    .y_axis_label("Networks");

                plot.show(ui, |plot_ui| {
                    plot_ui.bar_chart(chart);
                });
            });

            ui.add_space(8.0);

            // Detail table
            theme::card_frame().show(ui, |ui| {
                ui.label(RichText::new("Channel Details").size(14.0).color(theme::TEXT_PRIMARY).strong());
                ui.add_space(8.0);

                let active_channels: Vec<&ChannelInfo> = channels
                    .iter()
                    .filter(|ch| ch.network_count > 0 || ch.is_recommended)
                    .collect();

                egui_extras::TableBuilder::new(ui)
                    .striped(true)
                    .resizable(true)
                    .cell_layout(Layout::left_to_right(Align::Center))
                    .column(egui_extras::Column::exact(70.0))   // Channel
                    .column(egui_extras::Column::exact(90.0))   // Frequency
                    .column(egui_extras::Column::exact(80.0))   // Networks
                    .column(egui_extras::Column::exact(90.0))   // Avg Signal
                    .column(egui_extras::Column::exact(90.0))   // Congestion
                    .column(egui_extras::Column::remainder())     // SSIDs
                    .header(24.0, |mut header| {
                        header.col(|ui| { ui.label(RichText::new("Channel").size(11.0).color(theme::TEXT_MUTED).strong()); });
                        header.col(|ui| { ui.label(RichText::new("Frequency").size(11.0).color(theme::TEXT_MUTED).strong()); });
                        header.col(|ui| { ui.label(RichText::new("Networks").size(11.0).color(theme::TEXT_MUTED).strong()); });
                        header.col(|ui| { ui.label(RichText::new("Avg Signal").size(11.0).color(theme::TEXT_MUTED).strong()); });
                        header.col(|ui| { ui.label(RichText::new("Congestion").size(11.0).color(theme::TEXT_MUTED).strong()); });
                        header.col(|ui| { ui.label(RichText::new("Networks").size(11.0).color(theme::TEXT_MUTED).strong()); });
                    })
                    .body(|body| {
                        body.rows(24.0, active_channels.len(), |mut row| {
                            let ch = active_channels[row.index()];

                            row.col(|ui| {
                                let text = if ch.is_recommended {
                                    RichText::new(format!("⭐ {}", ch.channel)).size(12.0).color(theme::SUCCESS).strong()
                                } else {
                                    RichText::new(format!("{}", ch.channel)).size(12.0).color(theme::TEXT_PRIMARY)
                                };
                                ui.label(text);
                            });
                            row.col(|ui| {
                                ui.label(RichText::new(format!("{} MHz", ch.frequency)).size(11.0).color(theme::TEXT_SECONDARY));
                            });
                            row.col(|ui| {
                                ui.label(RichText::new(format!("{}", ch.network_count)).size(12.0).color(theme::TEXT_PRIMARY));
                            });
                            row.col(|ui| {
                                if ch.network_count > 0 {
                                    ui.label(RichText::new(format!("{:.0} dBm", ch.avg_signal)).size(11.0).color(theme::TEXT_SECONDARY));
                                } else {
                                    ui.label(RichText::new("—").size(11.0).color(theme::TEXT_MUTED));
                                }
                            });
                            row.col(|ui| {
                                let (text, badge) = match ch.congestion {
                                    CongestionLevel::Low => ("Low", BadgeType::Success),
                                    CongestionLevel::Medium => ("Medium", BadgeType::Warning),
                                    CongestionLevel::High => ("High", BadgeType::Error),
                                };
                                status_badge::status_badge(ui, text, badge);
                            });
                            row.col(|ui| {
                                let ssids = ch.networks.join(", ");
                                let display = if ssids.len() > 40 {
                                    format!("{}...", &ssids[..40])
                                } else if ssids.is_empty() {
                                    "—".to_string()
                                } else {
                                    ssids
                                };
                                ui.label(RichText::new(display).size(10.0).color(theme::TEXT_MUTED));
                            });
                        });
                    });
            });
        } else {
            // Empty state
            theme::card_frame().show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(30.0);
                    ui.label(RichText::new("📊").size(40.0));
                    ui.add_space(8.0);
                    ui.label(RichText::new("No analysis yet").size(14.0).color(theme::TEXT_SECONDARY));
                    ui.label(RichText::new("First scan WiFi networks, then click 'Analyze Channels'").size(12.0).color(theme::TEXT_MUTED));
                    ui.add_space(30.0);
                });
            });
        }
    });
}

fn perform_analysis(state: &mut ChannelAnalyzerState, networks: &[WifiNetwork]) {
    if networks.is_empty() {
        // Try scanning first
        match crate::wifi::scan_wifi() {
            Ok(nets) => {
                if nets.is_empty() {
                    state.error = Some("No WiFi networks found. Make sure WiFi is enabled.".to_string());
                    return;
                }
                state.analysis = Some(analyze_channels(&nets));
                state.error = None;
            }
            Err(e) => {
                state.error = Some(format!("WiFi scan failed: {}", e));
            }
        }
    } else {
        state.analysis = Some(analyze_channels(networks));
        state.error = None;
    }
}
