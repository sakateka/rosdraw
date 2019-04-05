use nannou::prelude::*;
use nannou::ui::prelude::*;
use crate::station::Station;

pub struct Model {
    pub ui: Ui,
    pub ids: Ids,
    pub mining: f32,
    pub burning: f32,
    pub shipping: f32,
    pub stations: [Station; 3],
    pub position: Point2,
}

impl Model {
    pub fn build(&mut self) {
        let ui = &mut self.ui.set_widgets();

        // Controls
        self.mining = Self::build_slider(self.mining)
            .top_left_with_margin(20.0)
            .label("Mining")
            .set(self.ids.mining, ui).unwrap_or(self.mining);
        self.burning = Self::build_slider(self.burning)
            .down(10.0)
            .label("Burning")
            .set(self.ids.burning, ui).unwrap_or(self.burning);

        self.shipping = Self::build_slider(self.shipping)
            .down(10.0)
            .label("Shipping")
            .set(self.ids.shipping, ui).unwrap_or(self.shipping);


        // Stations
        Self::build_station(&mut self.stations[0])
            .top_right_with_margin(20.0)
            .set(self.ids.s[0], ui);
        Self::build_station(&mut self.stations[1])
            .left(20.0)
            .set(self.ids.s[1], ui);
        Self::build_station(&mut self.stations[2])
            .left(20.0)
            .set(self.ids.s[2], ui);

        // Mine
        //station(800.0, 1.0, 1000.0, 160.0)
         //   .bottom_left_with_margin(20.0)
          //  .label("800")
           // .set(self.ids.mine, ui);
    }

    fn build_slider(val: f32) -> widget::Slider<'static, f32> {
        widget::Slider::new(val, 1.0, 10.0)
            .w_h(200.0, 30.0)
            .label_font_size(15)
            .rgb(0.3, 0.3, 0.3)
            .label_rgb(1.0, 1.0, 1.0)
            .border(0.0)
    }

    fn build_station<'a>(s: &'a mut Station) -> widget::Slider<'a, f32> {
        let f: f32 = *s.fuel.lock().unwrap();
        s.label = format!("{}", f);
        widget::Slider::new(f, 0., s.capacity)
            .label(&s.label)
            .enabled(false)
            .w_h(s.height * 0.5, s.height)
            .label_font_size(15)
            .rgb(0.3, 0.8, 0.3)
            .label_rgb(1.0, 1.0, 1.0)
            .border(0.1)
    }

    fn build_vehicle(&mut self, ui: &mut UiCell) {
        widget::Rectangle::fill([40.0, 40.0])
            .rgb(0.4, 0.4, 0.4)
            .right(10.0)
            .align_bottom()
            .set(self.ids.vehicle, ui);
    }
}

pub struct Ids {
    pub mining: widget::Id,
    pub burning: widget::Id,
    pub shipping: widget::Id,
    pub s: [widget::Id; 3],
    pub mine: widget::Id,
    pub vehicle: widget::Id,
}

pub fn app(app: &App) -> Model {
    app.new_window()
       .with_title("Power station")
       .with_dimensions(600, 400)
       .build()
       .unwrap();


    // Set the loop mode to wait for events, an energy-efficient option for pure-GUI apps.
    app.set_loop_mode(LoopMode::rate_fps(30.0));

    // Create the UI.
    let mut ui = app.new_ui()
        .build()
        .unwrap();

    // Generate some ids for our widgets.
    let ids = Ids {
        mining: ui.generate_widget_id(),
        burning: ui.generate_widget_id(),
        shipping: ui.generate_widget_id(),
        s: [ui.generate_widget_id(), ui.generate_widget_id(), ui.generate_widget_id()],
        mine: ui.generate_widget_id(),
        vehicle: ui.generate_widget_id(),
    };

    // Init our variables
    let mining = 5.0;
    let burning = 5.0;
    let shipping = 3.0;
    let position = pt2(0.0, 0.0);
    let stations = [Station::new(1), Station::new(2), Station::new(3)];

    Model {
        ui,
        ids,
        mining,
        burning,
        shipping,
        stations,
        position,
    }
}

