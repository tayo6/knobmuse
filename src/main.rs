use eframe::egui::{self, Color32, Pos2, Shape, Stroke, Vec2, Sense, epaint::PathShape};
use std::f32::consts::TAU;

fn lerp_color(t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    if t <= 0.5 {
        let k = t * 2.0;
        Color32::from_rgb(
            (34.0 + (250.0-34.0)*k) as u8,
            (197.0 + (204.0-197.0)*k) as u8,
            (94.0 + (21.0-94.0)*k) as u8,
        )
    } else {
        let k = (t-0.5)*2.0;
        Color32::from_rgb(
            (250.0 + (220.0-250.0)*k) as u8,
            (204.0 + (38.0-204.0)*k) as u8,
            (21.0 + (38.0-21.0)*k) as u8,
        )
    }
}

fn angle_to_pos(c: Pos2, r: f32, a: f32) -> Pos2 {
    Pos2::new(c.x + a.cos()*r, c.y + a.sin()*r)
}
fn arc_points(c: Pos2, r: f32, s: f32, e: f32, n: usize) -> Vec<Pos2> {
    (0..=n).map(|i| {
        let t = i as f32 / n as f32;
        angle_to_pos(c, r, s + (e-s)*t)
    }).collect()
}

#[derive(Default)]
struct GradientKnobApp { value: f32, pulse_t: f32 }

impl eframe::App for GradientKnobApp {
    fn update(&mut self, ctx: &egui::Context, _f: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let desired = Vec2::splat(220.0);
            let (rect, resp) = ui.allocate_exact_size(desired, Sense::click_and_drag);
            let center = rect.center();
            let radius = 82.0_f32;
            let stroke_w = 15.0_f32;
            const START: f32 = 120.0_f32 * std::f32::consts::PI / 180.0;
            const END: f32 = 420.0_f32 * std::f32::consts::PI / 180.0;
            const SWEEP: f32 = 300.0_f32 * std::f32::consts::PI / 180.0;
            let grey = Color32::from_rgb(209,213,219);

            if resp.dragged() {
                if let Some(p) = resp.interact_pointer_pos() {
                    let v = p - center;
                    let mut ang = v.angle(); // -PI..PI
                    if ang < 0.0 { ang += TAU; }
                    let deg = ang.to_degrees();
                    let new_val = if deg >= 120.0 { (deg-120.0)/300.0*100.0 }
                                  else if deg <= 60.0 { (deg+240.0)/300.0*100.0 }
                                  else if deg < 90.0 { 0.0 } else { 100.0 };
                    self.value = new_val.clamp(0.0,100.0);
                    self.pulse_t = 1.0;
                }
            }
            let cur_t = self.value/100.0;
            let cur_ang = START + cur_t * SWEEP;
            let cur_col = lerp_color(cur_t);
            let painter = ui.painter_at(rect);

            // inactive grey
            if cur_ang < END - 0.001 {
                let pts = arc_points(center, radius, cur_ang, END, 48);
                painter.add(Shape::Path(PathShape{ points: pts, closed:false, fill: Color32::TRANSPARENT, stroke: Stroke::new(stroke_w, grey)}));
            }
            // active gradient
            if self.value > 0.1 {
                let steps = 75;
                for i in 0..steps {
                    let a0 = START + i as f32/steps as f32 * (cur_ang-START);
                    let a1 = START + (i+1) as f32/steps as f32 * (cur_ang-START);
                    let tmid = (a0+a1)*0.5 - START;
                    let col = lerp_color(tmid / SWEEP);
                    painter.line_segment([angle_to_pos(center,radius,a0), angle_to_pos(center,radius,a1)], Stroke::new(stroke_w, col));
                }
                painter.circle_filled(angle_to_pos(center,radius,START), stroke_w*0.5, lerp_color(0.0));
                painter.circle_filled(angle_to_pos(center,radius,cur_ang), stroke_w*0.5, cur_col);
            }
            // ticks 41
            for i in 0..=40 {
                let t = i as f32/40.0;
                let ang = START + t*SWEEP;
                let base_len = if i%10==0 {14.0} else if i%5==0 {10.0} else {6.0};
                let dist = (t-cur_t).abs();
                let scale = if dist<0.12 {1.0+(0.12-dist)/0.12*0.7} else {1.0};
                let len = base_len*scale;
                let col = if t<=cur_t { lerp_color(t) } else { grey };
                let p1 = angle_to_pos(center,96.0,ang);
                let p2 = angle_to_pos(center,96.0+len,ang);
                painter.line_segment([p1,p2], Stroke::new(if i%10==0 {2.6} else {1.4}*scale, col));
            }
            // knob
            let pulse = self.pulse_t.clamp(0.0,1.0);
            let r = 13.0 + pulse*1.5;
            let pos = angle_to_pos(center,radius,cur_ang);
            painter.circle_filled(pos, r+3.0, cur_col.linear_multiply(0.25));
            painter.circle_filled(pos, r, Color32::WHITE);
            painter.circle_stroke(pos, r, Stroke::new(3.0, cur_col));
            self.pulse_t -= ctx.input(|i| i.unstable_dt) * 12.5;
            ui.allocate_space(desired);
        });
    }
}

#[cfg(not(target_arch="wasm32"))]
fn main() -> eframe::Result<()> {
    let opts = eframe::NativeOptions{
        viewport: egui::ViewportBuilder::default().with_inner_size([360.0,420.0]),
       ..Default::default()
    };
    eframe::run_native("Gradient Knob", opts, Box::new(|_| Box::new(GradientKnobApp::default())))
}
#[cfg(target_arch="wasm32")]
fn main(){
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();
    let web_options = eframe::WebOptions::default();
    wasm_bindgen_futures::spawn_local(async move{
        eframe::WebRunner::new().start("the_canvas_id", web_options, Box::new(|_| Box::new(GradientKnobApp::default()))).await.expect("fail");
    });
}