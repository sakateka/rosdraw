use std::sync::{Mutex, Arc};
use std::thread;
use std::time::Duration;
use nannou::ui::prelude::*;

pub struct Station {
    id: widget::Id,
    id_burning: widget::Id,
    idx: usize,
    fuel: Arc<Mutex<f32>>,
    speed:  Arc<Mutex<f32>>,
    speed_update: f32,
    label: String,
    capacity: f32,
    height: f64,
}

impl Station {
    pub fn new(idx: usize, id: widget::Id, id_burning: widget::Id) -> Self {
        let fuel = Arc::new(Mutex::new(100.)); 
        let speed_update = 1.0;
        let speed = Arc::new(Mutex::new(speed_update));
        Self::launch(idx, fuel.clone(), speed.clone());
        Station{
            id,
            id_burning,
            idx,
            fuel,
            speed,
            speed_update,
            label: "0".to_string(),
            capacity: 100.0,
            height: 100.0,
        }
    }

    pub fn update(&mut self, ui: &mut UiCell) {
        let speed = self.build_control(ui, self.speed_update);
        if self.speed_update != speed {
            // avoid excess locks
            self.speed_update = speed;
            *self.speed.lock().unwrap() = speed;
        }
        let f: f32 = *self.fuel.lock().unwrap();
        self.label = format!("{:.0}", f);
        widget::Slider::new(f, 0., self.capacity)
            .label(&self.label)
            .enabled(false)
            .w_h(self.height * 0.5, self.height)
            .label_font_size(15)
            .rgb(0.3, 0.8, 0.3)
            .label_rgb(1.0, 1.0, 1.0)
            .border(0.1)
            .left(5.0)
            .set(self.id, ui);
    }

    pub fn build_control(&mut self, ui: &mut UiCell, speed: f32) -> f32 {
        let mut w = widget::Slider::new(speed, 0.0, 10.0)
            .w_h(10.0, 50.0)
            .rgb(1.0, 1.0, 0.3)
            .border(0.0);
        if self.idx == 0 {
            w = w.top_right_with_margin(20.0);
        } else {
            w = w.left(20.0);
        }
        w.set(self.id_burning, ui).unwrap_or(self.speed_update)
    }

    fn launch(id: usize, f: Arc<Mutex<f32>>, s: Arc<Mutex<f32>>) {
        thread::spawn(move ||{
            info!("Build station #{}", id);
            loop {
                thread::sleep(Duration::from_secs(1));
                if let Ok(mut f) = f.lock() {
                    if let Ok(s) = s.lock() {
                        if *f > 0. {
                            trace!("Station #{} burned {:.3} fuel", id, *s);
                            *f = f32::max(0., *f - *s);
                        }
                    }
                }
            }
        });
    }
        
}
