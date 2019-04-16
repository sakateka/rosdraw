#[macro_use]
extern crate log;
#[macro_use]
extern crate conrod;

use env_logger;
use model::{model, Model};
use nannou;
use nannou::event::SimpleWindowEvent;
use nannou::prelude::*;
use std::env;

mod mine;
mod model;
mod posixmq;
mod station;
mod tank;
mod vehicle;

fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    env::set_var(
        "RUST_LOG",
        env::var("RUST_LOG").unwrap_or("rosdraw=info".to_owned()),
    );
    env_logger::init();

    posixmq::cleanup_posix_queues();

    nannou::app(model, event, view).run();
}

fn event(_app: &App, mut m: Model, event: Event) -> Model {
    match event {
        Event::Update(_update) => {
            m.update();
        }
        Event::WindowEvent {
            simple: Some(SimpleWindowEvent::Resized(_)),
            ..
        } => {
            m.vehicle.resize();
        }

        Event::WindowEvent {
            simple: Some(SimpleWindowEvent::KeyPressed(nannou::VirtualKeyCode::Space)),
            ..
        } => {
            m.toggle_freeze();
        }

        _ => (),
    }
    m
}

fn view(app: &App, model: &Model, frame: Frame) -> Frame {
    let draw = app.draw();
    draw.background().rgb(0.02, 0.02, 0.02);

    model.vehicle.draw(&draw);

    draw.to_frame(app, &frame).unwrap();
    model.ui.draw_to_frame(app, &frame).unwrap();
    frame
}
