use std::sync::{Mutex, Arc};
use std::thread;
use std::time::Duration;

pub struct Station {
    pub fuel: Arc<Mutex<f32>>,
    pub label: String,
    pub capacity: f32,
    pub height: f64,
}

impl Station {
    pub fn new(id: usize) -> Self {
        let fuel = Arc::new(Mutex::new(100.)); 
        Self::launch(id, fuel.clone());
        Station{
            fuel: fuel,
            label: "0".to_string(),
            capacity: 100.0,
            height: 100.0,
        }
    }
    fn launch(id: usize, f: Arc<Mutex<f32>>) {
        thread::spawn(move ||{
            loop {
                thread::sleep(Duration::from_secs(1));
                if let Ok(mut f) = f.lock() {
                    if *f > 0. {
                        info!("#{} I burn fuel", id);
                        *f -= 1.;
                    }
                }
            }
        });
    }
        
}
