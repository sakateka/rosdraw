#[macro_use]
extern crate log;

use std::sync::{Arc, Mutex};
use std::thread;
use nannou;
use env_logger;
use nannou::prelude::*;
use std::{env, process};
use std::time::Duration;

pub struct Model {
    pub circles: Vec<Circle>,
    pub color: Rgba,
    pub update_period: f64,
    pub position: Arc<Mutex<f32>>,
    pub end: Arc<Mutex<f32>>,
}
impl Model {
    pub fn launch(self) -> Self {
        let p = self.position.clone();
        let e = self.end.clone();
        let delay = Duration::from_millis(100);
        thread::spawn(move || {
            loop {
                thread::sleep(delay);
                let pos = *p.lock().unwrap() - 0.5;
                let end = *e.lock().unwrap();
                info!("end {}, pos {}", end, pos);
                *p.lock().unwrap() = pos;
                if end > 0.0 && pos.abs() > end {
                    break;
                }
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
    pub end: Arc<Mutex<f32>>,
    pub x: f32,
    pub y: f32,
}

impl Circle {
    pub fn new(num: f32) -> Self {
        Circle {
            num,
            speed: Arc::new(Mutex::new(10.0)),
            position: Arc::new(Mutex::new(0.0)),
            end: Arc::new(Mutex::new(0.0)),
            x: 0.0,
            y: 0.0,
        }.launch()
    }
    pub fn launch(self) -> Self {
        let speed = self.speed.clone();
        let position = self.position.clone();
        let end = self.end.clone();
        let num = self.num;
        let delay = Duration::from_millis(100);
        info!("Spawn thread for circle #{}", self.num);
        thread::spawn(move || {
            loop {
                thread::sleep(delay);
                let pos = *position.lock().unwrap() + *speed.lock().unwrap() * num / 10.0;
                let end = *end.lock().unwrap();
                info!("cyrcle end {}, pos {}", end, pos);
                *position.lock().unwrap() = pos;
                if end > 0.0 && pos > 0.0 && pos.abs() > end {
                    break;
                }
            }
        });
        self
    }

    pub fn set_end(&mut self, end: f32) {
        *self.end.lock().unwrap() = end;
    }
    pub fn set_speed(&mut self, speed: f32) {
        *self.speed.lock().unwrap() = speed;
    }
    pub fn update_xy(&mut self, p: Point2) {
        self.x = p.x + 25.0;
        self.y = p.y - self.num * 25.0 - 40.0;
    }
    pub fn xy(&self) -> Point2 {
        pt2(self.x + *self.position.lock().unwrap(), self.y)
    }
    pub fn draw(&self) -> bool {
        self.position.lock().unwrap().abs() < self.end.lock().unwrap().abs()
    }
}

fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    env::set_var("RUST_LOG", env::var("RUST_LOG").unwrap_or("info".into()));
    env_logger::init();
    nannou::app(model, event, view).run();
}

pub fn model(app: &App) -> Model {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
       error!("Usage: {} <number of circles>", args[0]);
       process::exit(1);
    }

    let num_circles = args[1].parse::<u32>().map_err(|e| {
       error!("Can't parse number of circles from '{}': {:?}", args[1], e);
       process::exit(1);
    }).unwrap();
    info!("Draw {} circles", num_circles);

    info!("Create window");
    app.new_window()
       .with_title("Power station")
       .with_dimensions(600, 400)
       .build()
       .unwrap();


    app.set_loop_mode(LoopMode::rate_fps(60.0));

    info!("Create circles");
    let circles = (1..(num_circles+1)).map(|n|{
        Circle::new(n as f32)
    }).collect();

    // Create the UI.
    info!("Create ui");
    app.new_ui().build().unwrap();
    let color = WHITE;
    let update_period = 1001.0; // trigger update
    let position = Arc::new(Mutex::new(10.0));
    let end = Arc::new(Mutex::new(0.0));
    info!("Create model");
    Model{ circles, color, update_period, position, end }.launch()
}

fn event(app: &App, mut m: Model, event: Event) -> Model {
    match event {
        Event::Update(update) => {
            m.update_period += update.since_last.ms();
            if m.update_period > 1000.0 {
                m.update_period = 0.0;
                let rec = app.window_rect();
                let tl = rec.top_left();
                let end = rec.w() - 10.0;
                m.set_end(rec.h() - 20.0);
                m.circles.iter_mut().for_each(|c| {
                    c.update_xy(tl);
                    c.set_end(end);
                });
                let circles_len = m.circles.len();
                m.circles.retain(|c| c.draw());
                info!("Retain {} circles", circles_len);

                if m.circles.len() != circles_len {
                    m.color = Rgba::new(
                        random_f32(),
                        random_f32(),
                        random_f32(),
                        255.0,
                    );
                }
            }
        },
        _ => (),
    }
    m
}


fn view(app: &App, model: &Model, frame: Frame) -> Frame {
    let draw = app.draw();
    draw.background().rgb(0.02, 0.02, 0.02);
    for circle in model.circles.iter() {
        draw.ellipse()
            .xy(circle.xy())
            .radius(10.0);
    }
    let tl = app.window_rect().top_left();
    if model.draw() {
        draw.rect()
            .x_y(tl.x + 20.0, tl.y + model.pos())
            .color(model.color)
            .w_h(25.0, 25.0);
    }
    draw.to_frame(app, &frame).unwrap();
    frame
}
