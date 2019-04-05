#[macro_use]
extern crate log;
use nannou;
use env_logger;
use nannou::prelude::*;
use app::Model;
use std::env;

mod station;
mod app;


fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    env::set_var(
        "RUST_LOG",
        env::var("RUST_LOG").unwrap_or("rosdraw=info".to_owned()),
    );
    env_logger::init();

    nannou::app(model, event, view).run();
}

fn event(_app: &App, mut model: Model, event: Event) -> Model {
    if let Event::Update(_update) = event {
        model.build();
    }
    model
}

fn model(app: &App) -> Model {
    app::app(app)
}

// Draw the state of your `Model` into the given `Frame` here.
fn view(app: &App, model: &Model, frame: Frame) -> Frame {
    // Begin drawing
    let draw = app.draw();
    draw.background().rgb(0.02, 0.02, 0.02);
    // Write the result of our drawing to the window's OpenGL frame.
    draw.to_frame(app, &frame).unwrap();
    // Draw the state of the `Ui` to the frame.
    model.ui.draw_to_frame(app, &frame).unwrap();

    // Return the drawn frame.
    frame
}
