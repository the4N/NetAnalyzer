// ─────────────────────────────────────────────────────────────
// NetAnalyzer - Status Badge Widget
// ─────────────────────────────────────────────────────────────

use eframe::egui::{self, Color32, CornerRadius, Vec2};
use crate::theme;

pub enum BadgeType {
    Success,
    Warning,
    Error,
    Info,
    Custom(Color32),
}

impl BadgeType {
    fn color(&self) -> Color32 {
        match self {
            BadgeType::Success => theme::SUCCESS,
            BadgeType::Warning => theme::WARNING,
            BadgeType::Error => theme::ERROR,
            BadgeType::Info => theme::INFO,
            BadgeType::Custom(c) => *c,
        }
    }
}

/// Show a colored pill badge with text
pub fn status_badge(ui: &mut egui::Ui, text: &str, badge_type: BadgeType) {
    let color = badge_type.color();
    let bg_color = Color32::from_rgba_premultiplied(
        color.r(),
        color.g(),
        color.b(),
        30,
    );

    egui::Frame::default()
        .fill(bg_color)
        .corner_radius(CornerRadius::same(12))
        .inner_margin(egui::Margin::symmetric(10, 3))
        .show(ui, |ui| {
            ui.colored_label(color, text);
        });
}

/// Show a connection status indicator (dot + text)
pub fn connection_status(ui: &mut egui::Ui, is_connected: bool) {
    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(Vec2::new(10.0, 10.0), egui::Sense::hover());
        let color = if is_connected { theme::SUCCESS } else { theme::ERROR };

        // Outer glow
        ui.painter().circle_filled(
            rect.center(),
            6.0,
            Color32::from_rgba_premultiplied(color.r(), color.g(), color.b(), 40),
        );
        // Inner dot
        ui.painter().circle_filled(rect.center(), 4.0, color);

        ui.label(
            egui::RichText::new(if is_connected { "Connected" } else { "Disconnected" })
                .color(color)
                .size(12.0),
        );
    });
}

/// Security type badge for WiFi networks
pub fn security_badge(ui: &mut egui::Ui, security: &str) {
    let (badge_type, display) = if security.contains("WPA3") {
        (BadgeType::Success, "WPA3")
    } else if security.contains("WPA2") {
        (BadgeType::Success, "WPA2")
    } else if security.contains("WPA") {
        (BadgeType::Warning, "WPA")
    } else if security.contains("WEP") {
        (BadgeType::Error, "WEP")
    } else if security.contains("Open") || security.is_empty() {
        (BadgeType::Error, "Open")
    } else {
        (BadgeType::Info, security)
    };

    status_badge(ui, display, badge_type);
}
