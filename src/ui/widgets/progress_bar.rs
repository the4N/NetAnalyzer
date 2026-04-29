// ─────────────────────────────────────────────────────────────
// NetAnalyzer - Animated Progress Bar Widget
// ─────────────────────────────────────────────────────────────

use eframe::egui::{self, Color32, CornerRadius, Vec2, Rect, Pos2, FontId, Align2};
use crate::theme;

pub struct AnimatedProgressBar {
    pub progress: f32,     // 0.0 - 1.0
    pub label: String,
    pub color: Color32,
}

impl AnimatedProgressBar {
    pub fn new(progress: f32, label: &str) -> Self {
        Self {
            progress: progress.clamp(0.0, 1.0),
            label: label.to_string(),
            color: theme::PRIMARY,
        }
    }

    pub fn color(mut self, color: Color32) -> Self {
        self.color = color;
        self
    }

    pub fn show(&self, ui: &mut egui::Ui) -> egui::Response {
        let height = 24.0;
        let available_width = ui.available_width();
        let (rect, response) = ui.allocate_exact_size(
            Vec2::new(available_width, height),
            egui::Sense::hover(),
        );

        let painter = ui.painter_at(rect);

        // Background bar
        painter.rect_filled(rect, CornerRadius::same(4), theme::BG_DARKEST);

        // Progress fill
        let fill_width = rect.width() * self.progress;
        if fill_width > 0.0 {
            let fill_rect = Rect::from_min_size(
                rect.min,
                Vec2::new(fill_width, height),
            );
            painter.rect_filled(fill_rect, CornerRadius::same(4), self.color);

            // Glow at the end
            if self.progress < 1.0 && self.progress > 0.01 {
                let glow_x = rect.min.x + fill_width;
                painter.rect_filled(
                    Rect::from_min_size(
                        Pos2::new(glow_x - 3.0, rect.min.y),
                        Vec2::new(6.0, height),
                    ),
                    CornerRadius::same(2),
                    Color32::from_rgba_premultiplied(
                        self.color.r(),
                        self.color.g(),
                        self.color.b(),
                        80,
                    ),
                );
            }
        }

        // Label text
        let text = format!("{} - {:.0}%", self.label, self.progress * 100.0);
        painter.text(
            rect.center(),
            Align2::CENTER_CENTER,
            &text,
            FontId::proportional(11.0),
            theme::TEXT_PRIMARY,
        );

        response
    }
}

/// Simple signal strength bar (for WiFi scanner)
pub fn signal_bar(ui: &mut egui::Ui, signal_percent: u32, width: f32) {
    let bar_count = 5;
    let bar_width = width / (bar_count as f32 * 2.0 - 1.0);
    let max_height = 16.0;

    let (rect, _) = ui.allocate_exact_size(
        Vec2::new(width, max_height),
        egui::Sense::hover(),
    );

    let painter = ui.painter_at(rect);
    let filled_bars = ((signal_percent as f32 / 100.0) * bar_count as f32).ceil() as usize;

    for i in 0..bar_count {
        let bar_height = max_height * (i + 1) as f32 / bar_count as f32;
        let x = rect.min.x + i as f32 * bar_width * 2.0;
        let y = rect.max.y - bar_height;

        let bar_rect = Rect::from_min_size(
            Pos2::new(x, y),
            Vec2::new(bar_width, bar_height),
        );

        let color = if i < filled_bars {
            theme::signal_color(-(90 - signal_percent as i32))
        } else {
            theme::BG_ELEVATED
        };

        painter.rect_filled(bar_rect, CornerRadius::same(2), color);
    }
}
