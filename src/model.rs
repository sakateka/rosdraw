use nannou::prelude::*;
use nannou::ui::prelude::*;
use crate::station::Station;
use crate::mine::Mine;
use crate::vehicle::Vehicle;

const NUM_STATIONS: usize = 4;

pub struct Model {
    pub ui: Ui,
    pub ids: Ids,
    pub mining: f32,
    pub burning: f32,
    pub shipping: f32,
    pub stations: [Station; NUM_STATIONS],
    pub mine: Mine,
    pub vehicle: Vehicle,
}

impl Model {
    pub fn update(&mut self, since_last: f64) {
        let ui = &mut self.ui.set_widgets();

        // Controls
        self.mining = Self::build_slider(self.mining, 20., "Mining")
            .top_left_with_margin(20.0)
            .set(self.ids.mining, ui).unwrap_or(self.mining);
        self.burning = Self::build_slider(self.burning, 10., "Burning")
            .down(10.0)
            .set(self.ids.burning, ui).unwrap_or(self.burning);

        self.shipping = Self::build_slider(self.shipping, 3., "Shipping")
            .down(10.0)
            .set(self.ids.shipping, ui).unwrap_or(self.shipping);


        // Mine
        self.mine.update(ui, self.mining);

        // Stations
        for station in self.stations.iter_mut() {
            station.update(ui, self.burning);
        }

        self.vehicle.update(ui, self.shipping / 100.0, since_last);
    }

    fn build_slider(val: f32, max: f32, label: &'static str) -> widget::Slider<'static, f32> {
        widget::Slider::new(val, 0.0, max)
            .label(label)
            .w_h(200.0, 30.0)
            .label_font_size(15)
            .rgb(0.3, 0.3, 0.3)
            .label_rgb(1.0, 1.0, 1.0)
            .border(0.0)
    }
}

widget_ids!{
    #[derive(Clone)]
    pub struct Ids {
        mining,
        burning,
        shipping,
        stations[],
        mine,
        vehicle,
    }
}

pub fn model(app: &App) -> Model {
    app.new_window()
       .with_title("Power station")
       .with_dimensions(600, 400)
       .build()
       .unwrap();


    app.set_loop_mode(LoopMode::rate_fps(60.0));

    // Create the UI.
    let mut ui = app.new_ui()
        .build()
        .unwrap();

    let mut ids = Ids::new(ui.widget_id_generator());
    ids.stations.resize(NUM_STATIONS, &mut ui.widget_id_generator());

    let mining = 5.0;
    let burning = 1.0;
    let shipping = 1.0;
    let stations = [
        // stations are drawn in reverse order
        // recover order here
        Station::new(0, ids.stations[3]),
        Station::new(1, ids.stations[2]),
        Station::new(2, ids.stations[1]),
        Station::new(3, ids.stations[0]),
    ];
    let mine = Mine::new(ids.mine);
    let vehicle = Vehicle::new(ids.clone());

    Model {
        ui,
        ids,
        mining,
        burning,
        shipping,
        stations,
        mine,
        vehicle,
    }
}

