use std::sync::{Mutex, Arc};
use std::thread;
use std::time::Duration;
use nannou::rand::random_f32;
use nannou::ui::prelude::*;

pub struct Mine {
    id: widget::Id,
    fuel: Arc<Mutex<f32>>,
    speed: Arc<Mutex<f32>>,
    label: String,
    capacity: f32,
    height: f64,
}

impl Mine {
    pub fn new(id: widget::Id) -> Self {
        let fuel = Arc::new(Mutex::new(10.)); 
        let speed = Arc::new(Mutex::new(1.)); 
        let capacity = 200.0;  // 2x station's capacity
        Self::launch(fuel.clone(), speed.clone(), capacity);
        Mine{
            id,
            fuel,
            speed,
            label: "0".to_string(),
            capacity: capacity,
            height: 200.0,
        }
    }

    pub fn update(&mut self, ui: &mut UiCell, speed: f32) {
        let f: f32 = *self.fuel.lock().unwrap();
        *self.speed.lock().unwrap() = speed;
        self.label = format!("{:.0}", f);
        widget::Slider::new(f, 0., self.capacity)
            .label(&self.label)
            .enabled(false)
            .w_h(self.height * 0.3, self.height)
            .label_font_size(20)
            .color(color::GREEN)
            .label_rgb(1.0, 1.0, 1.0)
            .border(0.3)
            .bottom_left_with_margin(20.0)
            .set(self.id, ui);
    }

    fn launch(f: Arc<Mutex<f32>>, s: Arc<Mutex<f32>>, capacity: f32) {
        thread::spawn(move ||{
            info!("Build Mine");
            loop {
                thread::sleep(Duration::from_secs(1));
                if let Ok(mut f) = f.lock() {
                    let portion = random_f32() * *s.lock().unwrap();
                    if *f < capacity {
                        trace!("Miner mine fuel {:.3}", portion);
                        *f = f32::min(*f + portion as f32, capacity);
                    }
                }
            }
        });
    }
        
}
