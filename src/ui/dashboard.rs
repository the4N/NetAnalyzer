// ─────────────────────────────────────────────────────────────
// NetAnalyzer - Dashboard View
// ─────────────────────────────────────────────────────────────

use eframe::egui::{self, RichText, Layout, Align};
use crate::app::{Page, AppState};
use crate::theme;
use crate::ui::widgets::status_badge;

pub fn show_dashboard(ui: &mut egui::Ui, state: &AppState, current_page: &mut Page) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        // Page header
        ui.horizontal(|ui| {
            ui.label(RichText::new("Dashboard").size(24.0).color(theme::TEXT_PRIMARY).strong());
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(
                    RichText::new(chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string())
                        .size(12.0)
                        .color(theme::TEXT_MUTED),
                );
            });
        });
        ui.add_space(16.0);

        // Top row: Network Status + Quick Actions
        ui.columns(2, |cols| {
            // Network Status Card
            theme::card_frame().show(&mut cols[0], |ui| {
                ui.label(RichText::new("🌐  Network Status").size(14.0).color(theme::TEXT_PRIMARY).strong());
                ui.add_space(12.0);

                status_badge::connection_status(ui, true);
                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    ui.label(RichText::new("Local IP:").size(12.0).color(theme::TEXT_SECONDARY));
                    ui.label(RichText::new(&state.local_ip).size(12.0).color(theme::TEXT_PRIMARY));
                });

                ui.horizontal(|ui| {
                    ui.label(RichText::new("Hostname:").size(12.0).color(theme::TEXT_SECONDARY));
                    ui.label(RichText::new(&state.hostname).size(12.0).color(theme::TEXT_PRIMARY));
                });

                ui.horizontal(|ui| {
                    ui.label(RichText::new("OS:").size(12.0).color(theme::TEXT_SECONDARY));
                    ui.label(RichText::new(&state.os_info).size(12.0).color(theme::TEXT_PRIMARY));
                });
            });

            // Quick Actions Card
            theme::card_frame().show(&mut cols[1], |ui| {
                ui.label(RichText::new("⚡  Quick Actions").size(14.0).color(theme::TEXT_PRIMARY).strong());
                ui.add_space(12.0);

                ui.vertical(|ui| {
                    if styled_button(ui, "🔍  Scan IP Range", theme::PRIMARY) {
                        *current_page = Page::IpScanner;
                    }
                    ui.add_space(4.0);
                    if styled_button(ui, "🔌  Scan Ports", theme::SECONDARY) {
                        *current_page = Page::PortScanner;
                    }
                    ui.add_space(4.0);
                    if styled_button(ui, "⚡  Run Speed Test", theme::ACCENT) {
                        *current_page = Page::SpeedTest;
                    }
                    ui.add_space(4.0);
                    if styled_button(ui, "📡  Scan WiFi Networks", theme::SUCCESS) {
                        *current_page = Page::WifiScanner;
                    }
                });
            });
        });

        ui.add_space(8.0);

        // Bottom row: Stats summary
        ui.columns(4, |cols| {
            stat_card(&mut cols[0], "🔍", "IP Scan", &format!("{} hosts found", state.last_ip_count), theme::PRIMARY);
            stat_card(&mut cols[1], "🔌", "Port Scan", &format!("{} ports open", state.last_port_count), theme::SECONDARY);
            stat_card(&mut cols[2], "⚡", "Speed Test", &state.last_speed_result, theme::ACCENT);
            stat_card(&mut cols[3], "📡", "WiFi", &format!("{} networks", state.last_wifi_count), theme::SUCCESS);
        });

        ui.add_space(16.0);

        // Traffic Graph
        theme::card_frame().show(ui, |ui| {
            ui.label(RichText::new("📈  Real-time Traffic (bytes/s)").size(14.0).color(theme::TEXT_PRIMARY).strong());
            ui.add_space(8.0);
            
            let mut rx_points = Vec::new();
            let mut tx_points = Vec::new();
            
            let now_sec = state.start_time.elapsed().as_secs_f64();
            
            for (t, rx, tx) in &state.traffic_history {
                let x = *t - now_sec; // Negative time for X axis
                rx_points.push([x, *rx]);
                tx_points.push([x, *tx]);
            }
            
            let rx_line = egui_plot::Line::new(rx_points)
                .color(theme::SUCCESS)
                .name("Download (RX)")
                .width(2.0);
                
            let tx_line = egui_plot::Line::new(tx_points)
                .color(theme::PRIMARY)
                .name("Upload (TX)")
                .width(2.0);

            egui_plot::Plot::new("traffic_plot")
                .height(180.0)
                .allow_drag(false)
                .allow_zoom(false)
                .include_y(0.0)
                .include_y(1024.0) // At least 1KB scale
                .include_x(-60.0)
                .include_x(0.0)
                .show_axes([false, true])
                .show(ui, |plot_ui| {
                    plot_ui.line(rx_line);
                    plot_ui.line(tx_line);
                });
        });

        ui.add_space(16.0);

        // Info box
        theme::card_frame().show(ui, |ui| {
            ui.label(RichText::new("ℹ️  Information").size(14.0).color(theme::TEXT_PRIMARY).strong());
            ui.add_space(8.0);
            ui.label(
                RichText::new("• Some features (ICMP Ping, WiFi Scan) may require administrator/root privileges")
                    .size(12.0)
                    .color(theme::TEXT_SECONDARY),
            );
            ui.label(
                RichText::new("• IP Scanner uses TCP fallback if ICMP is unavailable")
                    .size(12.0)
                    .color(theme::TEXT_SECONDARY),
            );
            ui.label(
                RichText::new("• Speed test connects to Cloudflare's speed test servers")
                    .size(12.0)
                    .color(theme::TEXT_SECONDARY),
            );
        });
    });
}

fn styled_button(ui: &mut egui::Ui, text: &str, color: egui::Color32) -> bool {
    let button = egui::Button::new(
        RichText::new(text)
            .size(13.0)
            .color(theme::TEXT_PRIMARY),
    )
    .fill(color.linear_multiply(0.15))
    .stroke(egui::Stroke::new(1.0, color.linear_multiply(0.4)))
    .corner_radius(egui::CornerRadius::same(6))
    .min_size(egui::Vec2::new(ui.available_width(), 32.0));

    ui.add(button).clicked()
}

fn stat_card(ui: &mut egui::Ui, icon: &str, title: &str, value: &str, color: egui::Color32) {
    theme::card_frame().show(ui, |ui| {
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(icon).size(22.0));
            ui.add_space(4.0);
            ui.label(RichText::new(title).size(11.0).color(theme::TEXT_MUTED));
            ui.add_space(2.0);
            ui.label(RichText::new(value).size(12.0).color(color).strong());
        });
    });
}
