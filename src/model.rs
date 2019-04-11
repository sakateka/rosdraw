use nannou::prelude::*;
use nannou::ui::prelude::*;
use crate::station::Station;
use crate::mine::Mine;
use crate::vehicle::Vehicle;

pub const NUM_STATIONS: usize = 4;

pub struct Model {
    pub ui: Ui,
    pub ids: Ids,
    pub mining: f32,
    pub shipping: f32,
    pub stations: [Station; NUM_STATIONS],
    pub mine: Mine,
    pub vehicle: Vehicle,
    freeze: bool,
}

impl Model {
    pub fn toggle_freeze(&mut self) {
        self.freeze = !self.freeze;
        if self.freeze {
            trace!("TODO: Freeze!");
        } else {
            trace!("TODO: Unfreeze!");
        }

    }
    pub fn update(&mut self) {
        let ui = &mut self.ui.set_widgets();

        // Controls
        self.mining = Self::build_slider(self.mining, 7., "Mining")
            .top_left_with_margin(20.0)
            .set(self.ids.mining, ui).unwrap_or(self.mining);
        self.shipping = Self::build_slider(self.shipping, 7., "Shipping")
            .down(10.0)
            .set(self.ids.shipping, ui).unwrap_or(self.shipping);

        // Mine
        self.mine.update_ui(ui, self.mining);
        // Stations
        for station in self.stations.iter_mut() {
            station.update(ui);
        }

        // update only after stations and mine
        self.vehicle.update(ui, self.shipping / 100.0);
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
        shipping,
        stations[],
        burning[],
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
    ids.burning.resize(NUM_STATIONS, &mut ui.widget_id_generator());

    let stations = [
        // stations are drawn in reverse order, recover order here
        Station::new(0, ids.stations[0], ids.burning[0]),
        Station::new(1, ids.stations[1], ids.burning[1]),
        Station::new(2, ids.stations[2], ids.burning[2]),
        Station::new(3, ids.stations[3], ids.burning[3]),
    ];
    assert_eq!(stations.len(), NUM_STATIONS);
    let mine = Mine::new(ids.mine);
    let vehicle = Vehicle::new(ids.clone());

    Model {
        ui,
        ids,
        mining: 2.0,
        shipping: 2.0,
        stations,
        mine,
        vehicle,
        freeze: false,
    }
}
