use std::sync::{Mutex, Arc};
use std::time::Duration;
use std::thread;

enum TankState {
    Load,
    Unload,
    Nop,
}

pub struct Tank {
    state: TankState,
    fuel: f32,
    capacity: f32,
    labor: f64,
    chunk: f32,
}

impl Tank {
    pub fn new() -> Self {
        let t = Tank {
            state: TankState::Load,
            fuel: 0.0,
            capacity: 20.0,
            labor: 0.0,
            chunk: 0.10,
        };
        //t.spawn_worker();
        t
    }

    /*
    fn spawn_worker(&mut self) {
        let capacity = self.capacity;
        thread::spawn(move ||{
            info!("Employ vehicle worker");
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
    */
    pub fn load(&mut self) {
        if self.fuel == self.capacity {
            self.stop_transfer();
            return;
        }
        self.state = TankState::Load;
        if self.labor > self.percentage() as f64 * 10.0 /*ms*/ {
            self.fuel = f32::min(self.capacity, self.fuel + self.capacity * self.chunk);
            trace!("Load fuel: {:.0}%", self.percentage());
        }
    }

    pub fn unload(&mut self) {
        if self.fuel == 0.0 {
            self.stop_transfer();
            return;
        }
        self.state = TankState::Unload;
        if self.labor > (100.0 - self.percentage()) as f64 * 10.0 /*ms*/ {
            self.fuel = (self.fuel -  self.capacity * self.chunk).round();
            trace!("Unload fuel: {:.0}%", self.percentage());
        }
    }

    pub fn transfer_fuel(&mut self, labor: f64) {
        if self.in_work() {
            self.labor += labor;
            match self.state {
                TankState::Load => self.load(),
                TankState::Unload => self.unload(),
                _ => (),
            };
        }
    }

    pub fn stop_transfer(&mut self) {
        self.state = TankState::Nop;
        self.labor = 0.0;
    }


    pub fn in_work(&self) -> bool {
        match self.state {
            TankState::Nop => false,
            _ => true,
        }
    }
    pub fn percentage(&self) -> f32 {
        (self.fuel / self.capacity * 100.0).round()
    }
}

