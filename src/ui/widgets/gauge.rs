// ─────────────────────────────────────────────────────────────
// NetAnalyzer - Circular Speed Gauge Widget
// ─────────────────────────────────────────────────────────────

use eframe::egui::{self, Color32, Pos2, Stroke, Vec2, FontId, Align2};
use std::f32::consts::PI;
use crate::theme;

pub struct SpeedGauge {
    pub value: f64,          // Current speed in Mbps
    pub max_value: f64,      // Max scale value
    pub label: String,       // "Download" or "Upload"
    pub unit: String,        // "Mbps"
    pub animated_value: f64, // For smooth animation
}

impl SpeedGauge {
    pub fn new(value: f64, max_value: f64, label: &str) -> Self {
        Self {
            value,
            max_value,
            animated_value: value,
            label: label.to_string(),
            unit: "Mbps".to_string(),
        }
    }

    pub fn show(&self, ui: &mut egui::Ui, size: f32) {
        let (rect, _response) = ui.allocate_exact_size(
            Vec2::new(size, size),
            egui::Sense::hover(),
        );

        let painter = ui.painter_at(rect);
        let center = rect.center();
        let radius = size * 0.4;

        // Background arc
        let start_angle = PI * 0.75;  // 135 degrees
        let end_angle = PI * 2.25;    // 405 degrees (270 degree sweep)
        let arc_width = 8.0;

        draw_arc(
            &painter,
            center,
            radius,
            start_angle,
            end_angle,
            arc_width,
            Color32::from_rgba_premultiplied(59, 130, 246, 25),
        );

        // Value arc (colored based on speed)
        let fraction = (self.animated_value / self.max_value).min(1.0) as f32;
        let value_end = start_angle + (end_angle - start_angle) * fraction;

        let gradient_color = speed_gradient_color(fraction);
        draw_arc(
            &painter,
            center,
            radius,
            start_angle,
            value_end,
            arc_width,
            gradient_color,
        );

        // Glow effect on the arc end
        if fraction > 0.01 {
            let glow_angle = value_end;
            let glow_x = center.x + radius * glow_angle.cos();
            let glow_y = center.y + radius * glow_angle.sin();
            painter.circle_filled(
                Pos2::new(glow_x, glow_y),
                6.0,
                gradient_color,
            );
            painter.circle_filled(
                Pos2::new(glow_x, glow_y),
                10.0,
                Color32::from_rgba_premultiplied(
                    gradient_color.r(),
                    gradient_color.g(),
                    gradient_color.b(),
                    40,
                ),
            );
        }

        // Scale markers
        let num_markers = 10;
        for i in 0..=num_markers {
            let t = i as f32 / num_markers as f32;
            let angle = start_angle + (end_angle - start_angle) * t;
            let inner = radius - 15.0;
            let outer = radius - 10.0;

            let p1 = Pos2::new(
                center.x + inner * angle.cos(),
                center.y + inner * angle.sin(),
            );
            let p2 = Pos2::new(
                center.x + outer * angle.cos(),
                center.y + outer * angle.sin(),
            );

            painter.line_segment([p1, p2], Stroke::new(1.5, theme::TEXT_MUTED));

            // Scale labels at key positions
            if i % 2 == 0 {
                let label_r = radius - 25.0;
                let label_pos = Pos2::new(
                    center.x + label_r * angle.cos(),
                    center.y + label_r * angle.sin(),
                );
                let val = (self.max_value * t as f64) as u32;
                painter.text(
                    label_pos,
                    Align2::CENTER_CENTER,
                    format!("{}", val),
                    FontId::proportional(9.0),
                    theme::TEXT_MUTED,
                );
            }
        }

        // Center value text
        painter.text(
            Pos2::new(center.x, center.y - 8.0),
            Align2::CENTER_CENTER,
            format!("{:.1}", self.animated_value),
            FontId::proportional(size * 0.14),
            theme::TEXT_PRIMARY,
        );

        // Unit text
        painter.text(
            Pos2::new(center.x, center.y + 14.0),
            Align2::CENTER_CENTER,
            &self.unit,
            FontId::proportional(11.0),
            theme::TEXT_SECONDARY,
        );

        // Label text
        painter.text(
            Pos2::new(center.x, center.y + radius + 20.0),
            Align2::CENTER_CENTER,
            &self.label,
            FontId::proportional(13.0),
            theme::TEXT_PRIMARY,
        );
    }
}

fn draw_arc(
    painter: &egui::Painter,
    center: Pos2,
    radius: f32,
    start: f32,
    end: f32,
    width: f32,
    color: Color32,
) {
    let segments = ((end - start).abs() * 30.0) as usize;
    let segments = segments.max(2);

    for i in 0..segments {
        let t1 = i as f32 / segments as f32;
        let t2 = (i + 1) as f32 / segments as f32;
        let a1 = start + (end - start) * t1;
        let a2 = start + (end - start) * t2;

        let p1 = Pos2::new(center.x + radius * a1.cos(), center.y + radius * a1.sin());
        let p2 = Pos2::new(center.x + radius * a2.cos(), center.y + radius * a2.sin());

        painter.line_segment([p1, p2], Stroke::new(width, color));
    }
}

fn speed_gradient_color(fraction: f32) -> Color32 {
    if fraction < 0.33 {
        theme::ERROR
    } else if fraction < 0.66 {
        theme::WARNING
    } else {
        theme::SUCCESS
    }
}
