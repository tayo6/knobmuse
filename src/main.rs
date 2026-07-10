use eframe::egui::{self, Color32, Pos2, Sense, Stroke, epaint::PathShape};
use std::f32::consts::TAU;

fn lerp_u8(a: u8, b: u8, t: f32) -> u8 {
    ((a as f32) * (1.0 - t) + (b as f32) * t).round().clamp(0.0, 255.0) as u8
}

fn lerp_color(t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    // green [34,197,94] -> yellow [250,204,21] -> red [220,38,38]
    if t < 0.5 {
        let lt = t * 2.0;
        Color32::from_rgb(
            lerp_u8(34, 250, lt),
            lerp_u8(197, 204, lt),
            lerp_u8(94, 21, lt),
        )
    } else {
        let lt = (t - 0.5) * 2.0;
        Color32::from_rgb(
            lerp_u8(250, 220, lt),
            lerp_u8(204, 38, lt),
            lerp_u8(21, 38, lt),
        )
    }
}

fn angle_to_pos(center: Pos2, r: f32, ang: f32) -> Pos2 {
    Pos2::new(center.x + ang.cos() * r, center.y + ang.sin() * r)
}

fn arc_points(center: Pos2, r: f32, start: f32, end: f32, segs: usize) -> Vec<Pos2> {
    let mut pts = Vec::with_capacity(segs + 1);
    for i in 0..=segs {
        let t = i as f32 / segs as f32;
        let a = start + (end - start) * t;
        pts.push(angle_to_pos(center, r, a));
    }
    pts
}

struct GradientKnobApp {
    value: f32,
    last_tick: i32,
    pulse_t: f32,
}

impl Default for GradientKnobApp {
    fn default() -> Self {
        Self {
            value: 42.0,
            last_tick: -1,
            pulse_t: 0.0,
        }
    }
}

impl eframe::App for GradientKnobApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(24.0);
                ui.heading("Gradient Knob");
                ui.label(format!("{:.0}%", self.value));
                ui.add_space(12.0);

                // fixed 220x220 interaction area
                let (rect, response) = ui.allocate_exact_size(
                    egui::vec2(220.0_f32, 220.0_f32),
                    Sense::drag(),
                );
                let center = rect.center();
                let r = 80.0_f32;

                // drag handling -> angle -> value
                if response.dragged() {
                    if let Some(pos) = response.interact_pointer_pos() {
                        let delta = pos - center;
                        let mut angle_deg = delta.y.atan2(delta.x).to_degrees();
                        // normalize 0..360, egui atan2 gives -180..180, with 0 = right
                        if angle_deg < 0.0 { angle_deg += 360.0; }

                        // Gap is 60deg centered at bottom: 60° to 120° is dead zone
                        // Active arc: 120° -> 420° (=60°) = 300° sweep
                        // We snap values that fall in gap to nearest edge
                        let snapped_deg = if (60.0_f32..120.0_f32).contains(&angle_deg) {
                            if angle_deg < 90.0 { 60.0_f32 } else { 120.0_f32 }
                        } else {
                            angle_deg
                        };

                        let norm_deg = if snapped_deg >= 120.0_f32 {
                            snapped_deg - 120.0_f32
                        } else {
                            // 0..60 maps to 240..300
                            snapped_deg + 240.0_f32
                        };

                        let new_value = (norm_deg / 300.0_f32 * 100.0_f32).clamp(0.0_f32, 100.0_f32);
                        let new_tick = (new_value / 2.5_f32).round() as i32; // 40 steps -> haptics
                        if new_tick != self.last_tick {
                            self.last_tick = new_tick;
                            self.pulse_t = 1.0_f32;
                            // optional haptic on web
                            #[cfg(target_arch = "wasm32")]
                            {
                                if let Some(win) = web_sys::window() {
                                    let _ = win.navigator().vibrate_with_duration(10);
                                }
                            }
                        }
                        self.value = new_value;
                    }
                }

                // derived state hoisted for whole scope (fixes current_t out of scope)
                let current_t = self.value / 100.0_f32;
                let current_angle_rad = 120.0_f32.to_radians() + current_t * 300.0_f32.to_radians();
                let current_color = lerp_color(current_t);
                let start_angle = 120.0_f32.to_radians();
                let end_angle = 420.0_f32.to_radians(); // 60 deg = 360+60

                let painter = ui.painter_at(rect);

                // 1) inactive track: from current to end (grey)
                if current_t < 0.999_f32 {
                    let inactive_pts = arc_points(center, r, current_angle_rad, end_angle, 48);
                    painter.add(PathShape {
                        points: inactive_pts,
                        closed: false,
                        fill: Color32::TRANSPARENT,
                        stroke: Stroke::new(15.0_f32, Color32::from_rgb(209, 213, 219)),
                    });
                }

                // 2) active gradient track: 75 segments from start to current
                let segs = 75usize;
                for i in 0..segs {
                    let t0 = i as f32 / segs as f32;
                    let t1 = (i + 1) as f32 / segs as f32;
                    // only draw up to current_t
                    if t0 > current_t { break; }
                    let ct1 = t1.min(current_t);
                    let global_t0 = t0; // global progress 0..1 corresponds to t along arc
                    // color by global position, not capped, to keep smooth gradient
                    let a0 = start_angle + t0 * 300.0_f32.to_radians();
                    let a1 = start_angle + ct1 * 300.0_f32.to_radians();
                    let p0 = angle_to_pos(center, r, a0);
                    let p1 = angle_to_pos(center, r, a1);
                    let col = lerp_color(global_t0);
                    painter.line_segment([p0, p1], Stroke::new(15.0_f32, col));
                }

                // round caps
                let cap_r = 7.5_f32;
                let start_pos = angle_to_pos(center, r, start_angle);
                painter.circle_filled(start_pos, cap_r, lerp_color(0.0_f32));
                let current_pos = angle_to_pos(center, r, current_angle_rad);
                painter.circle_filled(current_pos, cap_r, current_color);

                // 3) outer ticks (41 ticks)
                let tick_count = 41usize;
                let inner_r = 96.0_f32;
                let outer_r = 104.0_f32;
                for i in 0..tick_count {
                    let t = i as f32 / (tick_count as f32 - 1.0_f32);
                    let ang = start_angle + t * 300.0_f32.to_radians();
                    let dist = (t - current_t).abs();
                    let active = dist < 0.12_f32;
                    let scale = if active {
                        1.0_f32 + (0.12_f32 - dist) / 0.12_f32 * 0.7_f32
                    } else {
                        1.0_f32
                    };
                    let col = if active {
                        lerp_color(t)
                    } else {
                        Color32::from_rgb(156, 163, 175)
                    };
                    let len = (outer_r - inner_r) * scale;
                    let p_inner = angle_to_pos(center, inner_r, ang);
                    let p_outer = angle_to_pos(center, inner_r + len, ang);
                    painter.line_segment([p_inner, p_outer], Stroke::new(1.5_f32 * scale, col));
                }

                // 4) center knob
                let mut base_r = 13.0_f32;
                // pulse animation - fixed: unstable_dt is f32, not Option
                self.pulse_t -= ctx.input(|i| i.unstable_dt) * 12.5_f32; // 80ms decay
                if self.pulse_t > 0.0_f32 {
                    base_r += self.pulse_t * 3.0_f32;
                }
                let handle_pos = center;
                painter.circle_filled(handle_pos, base_r + 2.0_f32, current_color);
                painter.circle_filled(handle_pos, base_r, Color32::WHITE);
                painter.circle_stroke(handle_pos, base_r, Stroke::new(3.0_f32, current_color));

                // value text already above, but also draw inside for polish
                // (no extra scope issues)
            });
        });
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    let opts = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0_f32, 400.0_f32]),
        ..Default::default()
    };
    eframe::run_native(
        "Gradient Knob",
        opts,
        Box::new(|_cc| Box::new(GradientKnobApp::default())),
    )
}

#[cfg(target_arch = "wasm32")]
fn main() {
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();
    let web_options = eframe::WebOptions::default();
    wasm_bindgen_futures::spawn_local(async move {
        eframe::WebRunner::new()
            .start(
                "the_canvas_id",
                web_options,
                Box::new(|_cc| Box::new(GradientKnobApp::default())),
            )
            .await
            .expect("failed to start eframe");
    });
}
