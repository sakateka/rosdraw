use std::sync::{Mutex, Arc};
use std::thread;
use std::time::Duration;
use nannou::ui::prelude::*;
use crate::posixmq::{PMQ, Msg, VEHICLE_QUEUE, STATION_QUEUE_PREFIX};

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
        let fuel = Arc::new(Mutex::new(10.));
        let speed_update = 0.3;
        let speed = Arc::new(Mutex::new(speed_update));
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
        }.launch()
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
        let mut w = widget::Slider::new(speed, 0.0, 1.0)
            .w_h(10.0, 50.0)
            .rgb(1.0, 1.0, 0.3)
            .border(0.0);
        if self.idx == 0 {
            w = w.top_right_with_margin(20.0);
        } else {
            w = w.left(20.0);
        }
        w.set(self.id_burning, ui).unwrap_or(speed)
    }

    fn launch(self) -> Self {
        let idx = self.idx;
        let f = self.fuel.clone();
        let s = self.speed.clone();
        let capacity = self.capacity;
        let mq_v = PMQ::open(VEHICLE_QUEUE);
        let q_name = format!("{}{}", STATION_QUEUE_PREFIX, idx);
        let q = PMQ::open(q_name.as_ref()).nonblocking();
        let delay = Duration::from_millis(100);

        thread::spawn(move ||{
            info!("Build station #{}", idx);
            loop {
                let msg = q.receive();
                if let Ok(ref msg) = msg {
                    trace!("Station #{} receive msg: {:?}, current {}", idx, msg, *f.lock().unwrap());
                }
                match msg {
                    Ok(Msg::Fuel(amount)) => {
                        if amount > 0.0 {
                            // full station in blocking mode
                            q.set_nonblocking(false);

                            thread::sleep(delay / 2);
                            let mut f = f.lock().unwrap();
                            let update = *f + amount;
                            let remain = f32::max(update - capacity, 0.0);
                            *f = update - remain;
                            if remain > 0.0 {
                                info!("Set queue non blocking mode remain={}", remain);
                                mq_v.send(Msg::Fuel(remain)).expect("Send remain tank fuel");
                            } else {
                                mq_v.send(Msg::TankUnload).expect("Send tank unload");
                            }
                        } else {
                            mq_v.send(Msg::TankMove).expect("Send TankMove from station");
                            info!("Set queue non blocking mode");
                            q.set_nonblocking(true);
                        }
                    },
                    Ok(_) => unreachable!(),
                    Err(_) => {
                        if let Ok(mut f) = f.lock() {
                            if let Ok(s) = s.lock() {
                                if *f > 0.0 {
                                    if  *s > 0.0 {
                                        trace!("Station #{} burned {:.3} fuel", idx, *s);
                                        *f = f32::max(0., *f - *s);
                                    }
                                } else {
                                    mq_v.send(Msg::IdleStation(idx)).expect("Send idle station");
                                    info!("Set queue blocking mode");
                                    q.set_nonblocking(false);
                                }
                            }
                        }
                        thread::sleep(delay);
                    }
                }
            }
        });
        self
    }
}
