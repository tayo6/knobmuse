use eframe::egui::{self, Color32, Pos2, Shape, Stroke, Vec2, Sense, epaint::PathShape};
use std::f32::consts::TAU;

fn lerp_color(t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    if t <= 0.5 {
        let k = t * 2.0;
        Color32::from_rgb((34.0+(250.0-34.0)*k) as u8,(197.0+(204.0-197.0)*k) as u8,(94.0+(21.0-94.0)*k) as u8)
    } else {
        let k = (t-0.5)*2.0;
        Color32::from_rgb((250.0+(220.0-250.0)*k) as u8,(204.0+(38.0-204.0)*k) as u8,(21.0+(38.0-21.0)*k) as u8)
    }
}
fn angle_to_pos(c: Pos2, r: f32, a: f32) -> Pos2 { Pos2::new(c.x + a.cos()*r, c.y + a.sin()*r) }
fn arc_points(c: Pos2, r: f32, s: f32, e: f32, n: usize) -> Vec<Pos2> { (0..=n).map(|i|{let t=i as f32/n as f32; angle_to_pos(c,r,s+(e-s)*t)}).collect() }

#[derive(Default)]
struct App { v: f32, pulse: f32 }

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _f: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui|{
            let desired = Vec2::splat(220.0_f32);
            let (rect, resp) = ui.allocate_exact_size(desired, Sense::click_and_drag());
            let center = rect.center();
            let radius = 82.0_f32;
            let sw = 15.0_f32;
            const START: f32 = 120.0_f32*std::f32::consts::PI/180.0_f32;
            const END: f32 = 420.0_f32*std::f32::consts::PI/180.0_f32;
            const SWEEP: f32 = 300.0_f32*std::f32::consts::PI/180.0_f32;
            let grey = Color32::from_rgb(209,213,219);
            if resp.dragged(){
                if let Some(p)=resp.interact_pointer_pos(){
                    let dv = p-center;
                    let mut ang = dv.y.atan2(dv.x);
                    if ang<0.0 {ang+=TAU;}
                    let deg = ang.to_degrees();
                    let nv = if deg>=120.0 {(deg-120.0)/300.0*100.0} else if deg<=60.0 {(deg+240.0)/300.0*100.0} else if deg<90.0 {0.0} else {100.0};
                    self.v = nv.clamp(0.0,100.0);
                    self.pulse=1.0;
                }
            }
            let ct = self.v/100.0_f32;
            let ca = START+ct*SWEEP;
            let cc = lerp_color(ct);
            let painter = ui.painter_at(rect);
            if ca < END-0.001 {
                let pts = arc_points(center,radius,ca,END,48);
                painter.add(Shape::Path(PathShape{points:pts,closed:false,fill:Color32::TRANSPARENT,stroke:Stroke::new(sw,grey)}));
            }
            if self.v>0.1 {
                let steps=75;
                for i in 0..steps{
                    let a0=START+i as f32/steps as f32*(ca-START);
                    let a1=START+(i+1) as f32/steps as f32*(ca-START);
                    let tm=((a0+a1)*0.5-START)/SWEEP;
                    painter.line_segment([angle_to_pos(center,radius,a0),angle_to_pos(center,radius,a1)],Stroke::new(sw,lerp_color(tm)));
                }
                painter.circle_filled(angle_to_pos(center,radius,START),sw*0.5_f32,lerp_color(0.0));
                painter.circle_filled(angle_to_pos(center,radius,ca),sw*0.5_f32,cc);
            }
            for i in 0..=40{
                let t=i as f32/40.0_f32;
                let ang=START+t*SWEEP;
                let base=if i%10==0 {14.0_f32} else if i%5==0 {10.0_f32} else {6.0_f32};
                let dist=(t-ct).abs();
                let scale=if dist<0.12 {1.0+(0.12-dist)/0.12*0.7} else {1.0};
                let col=if t<=ct {lerp_color(t)} else {grey};
                let p1=angle_to_pos(center,96.0_f32,ang);
                let p2=angle_to_pos(center,96.0_f32+base*scale,ang);
                painter.line_segment([p1,p2],Stroke::new(if i%10==0 {2.6_f32} else {1.4_f32}*scale,col));
            }
            let r=13.0_f32+self.pulse.clamp(0.0,1.0)*1.5_f32;
            let pos=angle_to_pos(center,radius,ca);
            painter.circle_filled(pos,r+3.0_f32,Color32::from_black_alpha(40));
            painter.circle_filled(pos,r,Color32::WHITE);
            painter.circle_stroke(pos,r,Stroke::new(3.0_f32,cc));
            self.pulse-=ctx.input(|i| i.unstable_dt)*12.5_f32;
        });
    }
}
#[cfg(not(target_arch="wasm32"))]
fn main()->eframe::Result<()>{
    let opts=eframe::NativeOptions{viewport:egui::ViewportBuilder::default().with_inner_size([360.0,420.0]),..Default::default()};
    eframe::run_native("Gradient Knob",opts,Box::new(|_|Box::new(App::default())))
}
#[cfg(target_arch="wasm32")]
fn main(){
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();
    let wo=eframe::WebOptions::default();
    wasm_bindgen_futures::spawn_local(async move{eframe::WebRunner::new().start("the_canvas_id",wo,Box::new(|_|Box::new(App::default()))).await.expect("fail");});
}
