use crate::posixmq::{Msg, MINE_QUEUE, PMQ, VEHICLE_QUEUE};
use bytepack::{LEPacker, LEUnpacker};
use nannou::rand::random_f32;
use nannou::ui::prelude::*;
use nix::sys::stat;
use nix::unistd::{fork, getpid, getppid, mkfifo, ForkResult};
use nix::{errno::Errno, Error};
use std::fs::{File, OpenOptions};
use std::os::unix::fs::OpenOptionsExt;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{process, thread};

const MINE_PIPE: &'static str = "mine.pipe";

pub struct Mine {
    id: widget::Id,
    label: String,
    height: f64,
    fuel: Arc<Mutex<f32>>,
    speed_update: f32,
    speed: Arc<Mutex<f32>>,
    capacity: f32,
    pub q: PMQ,
}

impl Mine {
    pub fn new(id: widget::Id) -> Self {
        let capacity = 200.0; // 2x station's capacity
        let fuel = Arc::new(Mutex::new(0.0));
        let speed_update = 0.0;
        let speed = Arc::new(Mutex::new(speed_update));
        Self::launch(capacity, fuel.clone(), speed.clone());
        Mine {
            id,
            label: "0".to_string(),
            height: 200.0,
            fuel,
            speed_update,
            speed,
            capacity,
            q: PMQ::open(MINE_QUEUE),
        }
    }

    pub fn update_ui(&mut self, ui: &mut UiCell, speed: f32) {
        let fuel = *self.fuel.lock().unwrap();
        if self.speed_update != speed {
            self.speed_update = speed;
            *self.speed.lock().unwrap() = speed;
        }
        self.label = format!("{:.0}", fuel);
        widget::Slider::new(fuel, 0., self.capacity)
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

    fn mkfifo() {
        match mkfifo(MINE_PIPE, stat::Mode::S_IRWXU) {
            Ok(_) => info!("Created named pipe {:?}", MINE_PIPE),
            Err(Error::Sys(code)) => if code == Errno::EEXIST {},
            Err(err) => panic!("Error creating named pipe: {}", err),
        }
    }

    fn fork() {
        let delay = Duration::from_millis(100);
        Self::mkfifo();
        let parent_pid = getpid();
        match fork() {
            Ok(ForkResult::Parent { child, .. }) => {
                info!(
                    "Continuing execution in parent process, new child has pid: {}",
                    child
                );
            }
            Ok(ForkResult::Child) => {
                let mut pipe = Self::open_pipe_write();
                loop {
                    let portion = random_f32();
                    pipe.pack(portion).unwrap();
                    thread::sleep(delay);
                    if parent_pid != getppid() {
                        warn!("Parent pid exited, exit");
                        process::exit(1);
                    }
                }
            }
            Err(_) => panic!("Fork failed"),
        }
    }

    pub fn open_pipe_write() -> File {
        OpenOptions::new()
            .write(true)
            .append(true)
            .open(MINE_PIPE)
            .expect("Open mine pipe in write mode")
    }

    pub fn open_pipe_read() -> File {
        OpenOptions::new()
            .read(true)
            .custom_flags(nix::libc::O_NONBLOCK)
            .open(MINE_PIPE)
            .expect("Open mine pipe in read mode")
    }

    fn launch(capacity: f32, fuel: Arc<Mutex<f32>>, speed: Arc<Mutex<f32>>) {
        thread::spawn(move || {
            info!("Build Mine");
            let delay = Duration::from_millis(100);

            Self::mkfifo();
            Self::fork();
            let mut pipe = Self::open_pipe_read();
            let mq_m = PMQ::open(MINE_QUEUE).nonblocking();
            let mq_v = PMQ::open(VEHICLE_QUEUE);
            loop {
                let msg = mq_m.receive();
                if let Ok(ref msg) = msg {
                    trace!("Miner receive msg: {:?}", msg);
                }
                match msg {
                    Ok(Msg::Fuel(mut amount)) => {
                        let fourth = amount * 0.25;
                        amount = f32::min(amount, *fuel.lock().unwrap());
                        loop {
                            thread::sleep(delay / 2);
                            if amount <= 0.0 {
                                mq_v.send(Msg::TankMove).expect("Send move to vehicle");
                                break;
                            }
                            let val = f32::min(amount, fourth);
                            *fuel.lock().unwrap() -= val;
                            amount = f32::max(amount - val, 0.0);
                            mq_v.send(Msg::Fuel(val)).expect("Send fuel to vehicle");
                        }
                    }
                    Ok(_) => unreachable!("Unsupported message for Mine!"),
                    Err(_) => thread::sleep(delay),
                };
                match pipe.unpack::<f32>() {
                    Ok(portion) => {
                        let mut f = fuel.lock().unwrap();
                        let portion = portion * *speed.lock().unwrap();
                        if portion > 0.0 && *f < capacity {
                            *f = f32::min(*f + portion, capacity);
                            trace!("Miner mine fuel +val={:.3}, current={:.3}", portion, f);
                        }
                    }
                    Err(_) => (),
                }
            }
        });
    }
}
