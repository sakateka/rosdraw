#[macro_use]
extern crate log;
#[macro_use]
extern crate conrod;

use nannou;
use env_logger;
use nannou::prelude::*;
use nannou::event::SimpleWindowEvent;
use model::{model, Model};
use std::env;

mod model;
mod station;
mod mine;
mod vehicle;
mod tank;
mod posixmq;


fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    env::set_var(
        "RUST_LOG",
        env::var("RUST_LOG").unwrap_or("rosdraw=info".to_owned()),
    );
    env_logger::init();

    nannou::app(model, event, view).run();
}

fn event(_app: &App, mut m: Model, event: Event) -> Model {
    match event {
        Event::Update(update) => {
            m.update(update.since_last.ms());
        },
        Event::WindowEvent { simple: Some(SimpleWindowEvent::Resized(_)), .. } => {
            m.vehicle.resize();
        },

        Event::WindowEvent { simple: Some(SimpleWindowEvent::KeyPressed(nannou::VirtualKeyCode::Space)), .. } => {
            trace!("TODO: freeze");
            //m.toggle_freeze();
        },

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
