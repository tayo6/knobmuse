
use eframe::egui::{self, Color32, Pos2, Sense, Stroke, Vec2};
use std::f32::consts::{PI, TAU};

#[derive(Clone)]
struct GradientKnobApp {
    value: f32,
    dragging: bool,
    last_tick: i32,
    haptics_enabled: bool,
    pulse_t: f32,
    last_angle_deg: f32,
}

impl Default for GradientKnobApp {
    fn default() -> Self {
        Self {
            value: 42.0,
            dragging: false,
            last_tick: -1,
            haptics_enabled: true,
            pulse_t: 0.0,
            last_angle_deg: 120.0,
        }
    }
}

fn lerp_color(t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    if t < 0.5 {
        let k = t / 0.5;
        let g = [34u8, 197, 94];
        let y = [250u8, 204, 21];
        Color32::from_rgb(
            (g[0] as f32 + (y[0] as f32 - g[0] as f32) * k) as u8,
            (g[1] as f32 + (y[1] as f32 - g[1] as f32) * k) as u8,
            (g[2] as f32 + (y[2] as f32 - g[2] as f32) * k) as u8,
        )
    } else {
        let k = (t - 0.5) / 0.5;
        let y = [250u8, 204, 21];
        let r = [220u8, 38, 38];
        Color32::from_rgb(
            (y[0] as f32 + (r[0] as f32 - y[0] as f32) * k) as u8,
            (y[1] as f32 + (r[1] as f32 - y[1] as f32) * k) as u8,
            (y[2] as f32 + (r[2] as f32 - y[2] as f32) * k) as u8,
        )
    }
}

fn angle_to_value(angle_deg: f32) -> Option<f32> {
    // 60° gap at bottom: 60..120 = dead zone (7 to 5 o'clock)
    // Active: 120..420 (=> 0..360) mapped to 0..100
    if (60.0..120.0).contains(&angle_deg) {
        return None;
    }
    let v = if angle_deg >= 120.0 {
        (angle_deg - 120.0) / 300.0 * 100.0
    } else {
        // 0..60 wraps to 240..300 / 300
        (angle_deg + 240.0) / 300.0 * 100.0
    };
    Some(v.clamp(0.0, 100.0))
}

fn value_to_angle(value: f32) -> f32 {
    let t = (value / 100.0).clamp(0.0, 1.0);
    // TOTAL_START = 120deg, TOTAL_SWEEP = 300deg
    120.0 + t * 300.0
}

impl eframe::App for GradientKnobApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // pulse decay
        if self.pulse_t > 0.0 {
            self.pulse_t -= ctx.input(|i| i.unstable_dt).unwrap_or(1.0/60.0) * 12.5; // 80ms
            if self.pulse_t < 0.0 { self.pulse_t = 0.0; }
            ctx.request_repaint();
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(24.0);
                ui.heading("Gradient Knob — 180×180 • 15px stroke");
                ui.label(format!("value: {:.0}  •  haptics: {}", self.value, if self.haptics_enabled { "on" } else { "off" }));
                ui.checkbox(&mut self.haptics_enabled, "Haptic tick simulation");
                ui.add_space(16.0);

                // 220x220 allocate
                let (rect, response) = {
                    let size = Vec2::new(220.0, 220.0);
                    let (r, resp) = ui.allocate_exact_size(size, Sense::click_and_drag());
                    (r, resp)
                };

                const RADIUS: f32 = 82.0;
                const STROKE: f32 = 15.0;
                const TOTAL_START_DEG: f32 = 120.0; // 7:01 ~ 120°
                const TOTAL_END_DEG: f32 = 420.0;   // 4:59 ~ 60° (wrapped)
                const TOTAL_SWEEP_DEG: f32 = 300.0;

                let center = rect.center();
                let current_angle_deg = value_to_angle(self.value);
                let current_angle_rad = current_angle_deg.to_radians();
                let total_start_rad = TOTAL_START_DEG.to_radians();
                // let total_end_rad = TOTAL_END_DEG.to_radians();

                // --- pointer handling ---
                if response.drag_started() {
                    self.dragging = true;
                }
                if response.drag_stopped() {
                    self.dragging = false;
                }
                if self.dragging || response.dragged() {
                    if let Some(pointer_pos) = response.interact_pointer_pos().or_else(|| ctx.pointer_interact_pos()) {
                        let delta = pointer_pos - center;
                        let mut ang = delta.angle(); // -PI..PI, 0 = +X
                        if ang < 0.0 { ang += TAU; }
                        let mut deg = ang.to_degrees(); // 0..360
                        // egui Y down => atan2 matches canvas but 0° = east, CCW is positive? 
                        // We want same as web: 0° right, 90° down. emath angle() gives that.
                        // Snap gap
                        if (60.0..120.0).contains(&deg) {
                            // snap to nearest edge
                            deg = if deg < 90.0 { 60.0 } else { 120.0 };
                        }
                        if let Some(v) = angle_to_value(deg) {
                            let prev = self.value;
                            if (v - prev).abs() >= 1.0 {
                                let tick = v.round() as i32;
                                if tick != self.last_tick {
                                    if self.haptics_enabled && tick % 10 == 0 {
                                        self.pulse_t = 1.0; // strong tick
                                        // native haptics on mobile via winit:
                                        // ctx.output_mut(|o| o.commands.push(egui::OutputCommand::RequestUserAttention(...)))
                                        // or platform-specific: vibration API bridged via wasm
                                    } else if self.haptics_enabled {
                                        self.pulse_t = 0.45;
                                    }
                                    self.last_tick = tick;
                                }
                                self.value = v;
                            }
                            self.last_angle_deg = deg;
                        }
                    }
                }

                let painter = ui.painter_at(rect);

                // -- inactive track (from current to end) --
                {
                    let mut points_inactive = Vec::new();
                    let segs = 32;
                    let start = current_angle_rad;
                    let end = TOTAL_END_DEG.to_radians();
                    for i in 0..=segs {
                        let t = i as f32 / segs as f32;
                        let a = start + (end - start) * t;
                        points_inactive.push(Pos2::new(
                            center.x + a.cos() * RADIUS,
                            center.y + a.sin() * RADIUS,
                        ));
                    }
                    let shape = egui::epaint::PathShape {
                        points: points_inactive,
                        closed: false,
                        fill: Color32::TRANSPARENT,
                        stroke: egui::epaint::PathStroke::new(STROKE, Color32::from_hex("#d1d5db").unwrap_or(Color32::from_gray(200)))
                            .with_cap(egui::StrokeKind::Middle, egui::epaint::LineCap::Round),
                    };
                    painter.add(shape);
                }

                // -- active gradient arc: 75 segments --
                {
                    let segments = 75;
                    let start = total_start_rad;
                    let end = current_angle_rad;
                    let sweep = (end - start).rem_euclid(TAU);
                    for i in 0..segments {
                        let t0 = i as f32 / segments as f32;
                        let t1 = (i + 1) as f32 / segments as f32;
                        let a0 = start + sweep * t0;
                        let a1 = start + sweep * t1;
                        let p0 = Pos2::new(center.x + a0.cos() * RADIUS, center.y + a0.sin() * RADIUS);
                        let p1 = Pos2::new(center.x + a1.cos() * RADIUS, center.y + a1.sin() * RADIUS);
                        let col = lerp_color(t0);
                        painter.line_segment([p0, p1], Stroke::new(STROKE, col));
                    }
                    // round caps via circles
                    let c0 = lerp_color(0.0);
                    let c1 = lerp_color(self.value / 100.0);
                    let rcap = STROKE * 0.5;
                    let p_start = Pos2::new(center.x + total_start_rad.cos() * RADIUS, center.y + total_start_rad.sin() * RADIUS);
                    let p_end = Pos2::new(center.x + current_angle_rad.cos() * RADIUS, center.y + current_angle_rad.sin() * RADIUS);
                    painter.circle_filled(p_start, rcap, c0);
                    painter.circle_filled(p_end, rcap, c1);
                }

                // -- outer ticks 41 --
                {
                    let tick_count = 41;
                    let inner = 96.0;
                    let current_t = self.value / 100.0;
                    for i in 0..tick_count {
                        let t = i as f32 / (tick_count - 1) as f32;
                        let angle_deg = 120.0 + t * TOTAL_SWEEP_DEG;
                        let angle = angle_deg.to_radians();
                        let is_major = i % 10 == 0;
                        let is_med = i % 5 == 0;
                        let base_len = if is_major { 14.0 } else if is_med { 10.0 } else { 6.0 };

                        let dist = (t - current_t).abs();
                        let scale = if dist < 0.12 {
                            1.0 + (0.12 - dist) / 0.12 * 0.7
                        } else { 1.0 };

                        let len = base_len * scale;
                        let r0 = inner;
                        let r1 = inner + len;
                        let p0 = Pos2::new(center.x + angle.cos() * r0, center.y + angle.sin() * r0);
                        let p1 = Pos2::new(center.x + angle.cos() * r1, center.y + angle.sin() * r1);

                        let col = if t <= current_t + 0.001 { lerp_color(t) } else { Color32::from_gray(120) };
                        let width = if is_major { 2.2 } else { 1.4 } * scale;
                        painter.line_segment([p0, p1], Stroke::new(width, col));
                    }
                }

                // -- knob handle --
                {
                    let handle_pos = Pos2::new(
                        center.x + current_angle_rad.cos() * RADIUS,
                        center.y + current_angle_rad.sin() * RADIUS,
                    );
                    let scale = 1.0 + self.pulse_t * 0.08;
                    let base_r = 13.0 * scale;
                    // shadow
                    painter.circle_filled(handle_pos + Vec2::new(0.0, 2.0), base_r + 1.5, Color32::from_black_alpha(40));
                    painter.circle_filled(handle_pos, base_r, Color32::WHITE);
                    painter.circle_stroke(handle_pos, base_r, Stroke::new(3.0, lerp_color(current_t)));
                }

                // center text
                {
                    painter.text(
                        center,
                        egui::Align2::CENTER_CENTER,
                        format!("{:.0}", self.value),
                        egui::FontId::proportional(28.0),
                        Color32::WHITE,
                    );
                }
            });
        });
    }
}

fn main() -> eframe::Result<()> {
    let opts = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([360.0, 420.0])
            .with_min_inner_size([320.0, 360.0]),
        ..Default::default()
    };
    eframe::run_native(
        "gradient-knob",
        opts,
        Box::new(|_cc| Box::new(GradientKnobApp::default())),
    )
}