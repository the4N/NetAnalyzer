// ─────────────────────────────────────────────────────────────
// NetAnalyzer - Sidebar Navigation
// ─────────────────────────────────────────────────────────────

use eframe::egui::{self, RichText, CornerRadius, Vec2, Layout, Align};
use crate::app::Page;
use crate::theme;

pub fn show_sidebar(ui: &mut egui::Ui, current_page: &mut Page) {
    theme::sidebar_frame().show(ui, |ui| {
        ui.set_min_width(180.0);
        ui.set_max_width(180.0);

        ui.vertical(|ui| {
            // App title
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("⚡")
                        .size(22.0),
                );
                ui.label(
                    RichText::new("NetAnalyzer")
                        .size(18.0)
                        .color(theme::TEXT_PRIMARY)
                        .strong(),
                );
            });

            ui.add_space(4.0);
            ui.label(
                RichText::new("Network Analysis Tool")
                    .size(10.0)
                    .color(theme::TEXT_MUTED),
            );

            ui.add_space(16.0);
            ui.separator();
            ui.add_space(8.0);

            // Navigation items
            ui.label(
                RichText::new("MAIN")
                    .size(10.0)
                    .color(theme::TEXT_MUTED),
            );
            ui.add_space(4.0);

            nav_item(ui, "🏠", "Dashboard", Page::Dashboard, current_page);

            ui.add_space(12.0);
            ui.label(
                RichText::new("SCANNERS")
                    .size(10.0)
                    .color(theme::TEXT_MUTED),
            );
            ui.add_space(4.0);

            nav_item(ui, "🔍", "IP Scanner", Page::IpScanner, current_page);
            nav_item(ui, "🔌", "Port Scanner", Page::PortScanner, current_page);

            ui.add_space(12.0);
            ui.label(
                RichText::new("NETWORK")
                    .size(10.0)
                    .color(theme::TEXT_MUTED),
            );
            ui.add_space(4.0);

            nav_item(ui, "⚡", "Speed Test", Page::SpeedTest, current_page);
            nav_item(ui, "📡", "WiFi Scanner", Page::WifiScanner, current_page);
            nav_item(ui, "📊", "Channel Analyzer", Page::ChannelAnalyzer, current_page);

            // Fill remaining space
            ui.with_layout(Layout::bottom_up(Align::LEFT), |ui| {
                ui.add_space(8.0);
                ui.label(
                    RichText::new("v1.0")
                        .size(10.0)
                        .color(theme::TEXT_MUTED),
                );
                ui.separator();
            });
        });
    });
}

fn nav_item(ui: &mut egui::Ui, icon: &str, label: &str, page: Page, current_page: &mut Page) {
    let is_active = *current_page == page;

    let bg_color = if is_active {
        theme::PRIMARY.linear_multiply(0.15)
    } else {
        egui::Color32::TRANSPARENT
    };

    let text_color = if is_active {
        theme::PRIMARY
    } else {
        theme::TEXT_SECONDARY
    };

    let frame = egui::Frame::default()
        .fill(bg_color)
        .corner_radius(CornerRadius::same(6))
        .inner_margin(egui::Margin::symmetric(10, 6));

    let response = frame.show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        ui.horizontal(|ui| {
            ui.label(RichText::new(icon).size(14.0));
            ui.label(
                RichText::new(label)
                    .size(13.0)
                    .color(text_color),
            );

            // Active indicator
            if is_active {
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let (rect, _) = ui.allocate_exact_size(Vec2::new(3.0, 16.0), egui::Sense::hover());
                    ui.painter().rect_filled(rect, CornerRadius::same(2), theme::PRIMARY);
                });
            }
        });
    });

    if response.response.interact(egui::Sense::click()).clicked() {
        *current_page = page;
    }

    // Hover effect
    if response.response.interact(egui::Sense::hover()).hovered() && !is_active {
        ui.painter().rect_filled(
            response.response.rect,
            CornerRadius::same(6),
            theme::BG_HOVER.linear_multiply(0.5),
        );
    }
}
