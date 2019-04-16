#[macro_use]
extern crate log;

use env_logger;
use ipc_channel::ipc::{self, IpcReceiver, IpcSender};
use nannou;
use nannou::prelude::*;
use nannou::ui::prelude::{widget, Colorable, Positionable, Ui, Widget};
use procinfo::pid::stat_self;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use std::{env, process};

const PADDING: f32 = 25.0;

fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    env::set_var("RUST_LOG", env::var("RUST_LOG").unwrap_or("info".into()));
    env_logger::init();
    nannou::app(model, event, view).run();
}

pub fn model(app: &App) -> Model {
    info!("Create window");
    app.new_window()
        .with_multisampling(0)
        .with_title("Lab multithreading")
        .with_dimensions(600, 400)
        .build()
        .unwrap();

    app.set_loop_mode(LoopMode::rate_fps(60.0));

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        error!("Usage: {} <ipc channel name>", args[0]);
        process::exit(1);
    }
    let control = args[1].clone();
    let (tx, rx): (IpcSender<(usize, f32)>, IpcReceiver<(usize, f32)>) = ipc::channel().unwrap();
    let tx0 = IpcSender::connect(control).unwrap();
    info!("Send tx channel to control programm");
    tx0.send(tx).unwrap();
    let (num_circles, _) = rx.recv().expect("Receive num circles");
    info!("Received num circles: {}", num_circles);

    info!("Create {} circles", num_circles);
    let circles = (1..(num_circles as usize + 1))
        .map(|n| Circle::new(n as f32))
        .collect();

    // Create the UI.
    info!("Create ui");
    let ui = app.new_ui().build().unwrap();
    let color = WHITE;
    let update_period = 1001.0; // trigger update
    let position = Arc::new(Mutex::new(10.0));
    let end = Arc::new(Mutex::new(0.0));
    info!("Create model");

    Model {
        ui,
        circles,
        color,
        update_period,
        position,
        end,
        rx,
    }
    .launch()
}

fn event(app: &App, mut m: Model, event: Event) -> Model {
    match event {
        Event::Update(update) => {
            m.update_period += update.since_last.ms();
            m.update_period = 0.0;
            let rec = app.window_rect();
            let tl = rec.top_left();
            m.set_end(rec.h() - 20.0);
            m.circles.iter_mut().for_each(|c| {
                c.update_xy(tl);
                c.set_rect(rec);
            });
            let circles_len = m.circles.len();
            m.circles.retain(|c| c.draw());

            if m.circles.len() != circles_len {
                info!("Retain {} circles", m.circles.len());
                m.color = Rgba::new(random_f32(), random_f32(), random_f32(), 255.0);
                info!("Change cube color to {:?}", m.color);
            }
            match m.rx.try_recv() {
                Ok((idx, speed)) => {
                    if idx < 1 || idx > m.circles.len() {
                        error!(
                            "There are only {} circles (index from 1), but request speed change for {}",
                            m.circles.len(),
                            idx
                        )
                    } else {
                        info!("Change speed for circle={} to {}", idx, speed);
                        *m.circles[idx - 1].speed.lock().unwrap() = speed;
                    }
                }
                Err(_) => (),
            }
            let tr = rec.top_right();

            let text = match stat_self() {
                Ok(stat) => format!("{} threads", stat.num_threads).to_owned(),
                Err(_) => "".to_string(),
            };
            widget::Text::new(text.as_ref())
                .xy([tr.x as f64 - 40.0, tr.y as f64 - 20.0])
                .font_size(15)
                .color(conrod::color::ORANGE)
                .set(m.ui.generate_widget_id(), &mut m.ui.set_widgets());
        }
        _ => (),
    }
    m
}

fn view(app: &App, model: &Model, frame: Frame) -> Frame {
    let draw = app.draw();
    draw.background().rgb(0.02, 0.02, 0.02);
    for circle in model.circles.iter() {
        draw.ellipse().xy(circle.get_xy()).radius(10.0);
    }
    let tl = app.window_rect().top_left();
    if model.draw() {
        draw.rect()
            .x_y(tl.x + 20.0, tl.y + model.pos())
            .color(model.color)
            .w_h(25.0, 25.0);
    }
    draw.to_frame(app, &frame).unwrap();
    model.ui.draw_to_frame(app, &frame).unwrap();
    frame
}

pub struct Model {
    pub ui: Ui,
    pub circles: Vec<Circle>,
    pub color: Rgba,
    pub update_period: f64,
    pub position: Arc<Mutex<f32>>,
    pub end: Arc<Mutex<f32>>,
    pub rx: IpcReceiver<(usize, f32)>,
}
impl Model {
    pub fn launch(self) -> Self {
        let p = self.position.clone();
        let e = self.end.clone();
        let delay = Duration::from_millis(100);
        thread::spawn(move || loop {
            thread::sleep(delay);
            let pos = *p.lock().unwrap() - 0.5;
            let end = *e.lock().unwrap();
            *p.lock().unwrap() = pos;
            if end > 0.0 && pos.abs() > end {
                *p.lock().unwrap() = pos * 100.0;
                break;
            }
        });
        self
    }
    pub fn pos(&self) -> f32 {
        *self.position.lock().unwrap()
    }
    pub fn set_end(&mut self, end: f32) {
        *self.end.lock().unwrap() = end;
    }
    pub fn draw(&self) -> bool {
        self.position.lock().unwrap().abs() < self.end.lock().unwrap().abs()
    }
}

#[derive(Clone, Debug)]
pub struct Circle {
    pub num: f32,
    pub speed: Arc<Mutex<f32>>,
    pub position: Arc<Mutex<f32>>,
    pub rect: Arc<Mutex<Option<Rect>>>,
    pub draw: Arc<Mutex<bool>>,
    pub xy: Point2,
}

impl Circle {
    pub fn new(num: f32) -> Self {
        Circle {
            num,
            speed: Arc::new(Mutex::new(10.0)),
            position: Arc::new(Mutex::new(0.0)),
            rect: Arc::new(Mutex::new(None)),
            draw: Arc::new(Mutex::new(true)),
            xy: pt2(0.0, 0.0),
        }
        .launch()
    }
    pub fn launch(self) -> Self {
        let speed = self.speed.clone();
        let position = self.position.clone();
        let rect = self.rect.clone();
        let draw = self.draw.clone();
        let num = self.num;
        let delay = Duration::from_millis(100);
        info!("Spawn thread for circle #{}", self.num);
        thread::spawn(move || loop {
            thread::sleep(delay);
            let pos = *position.lock().unwrap() + *speed.lock().unwrap() / 10.0;
            *position.lock().unwrap() = pos;
            if let Some(r) = *rect.lock().unwrap() {
                let y_pos = num * 25.0 + PADDING;
                if pos + PADDING > r.w() || pos + PADDING < 0.0 || y_pos > r.h() {
                    *draw.lock().unwrap() = false;
                    *position.lock().unwrap() = pos * 100.0;
                    break;
                }
            }
        });
        self
    }

    pub fn set_rect(&mut self, rect: Rect) {
        *self.rect.lock().unwrap() = Some(rect);
    }
    pub fn set_speed(&mut self, speed: f32) {
        *self.speed.lock().unwrap() = speed;
    }
    pub fn update_xy(&mut self, p: Point2) {
        self.xy = pt2(p.x + PADDING, p.y - self.num * 25.0 - PADDING)
    }
    pub fn get_xy(&self) -> Point2 {
        pt2(self.xy.x + *self.position.lock().unwrap(), self.xy.y)
    }
    pub fn draw(&self) -> bool {
        *self.draw.lock().unwrap()
    }
}
